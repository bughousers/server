// Copyright (C) 2020  Kerem Çakırer

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use futures::channel::{mpsc, oneshot};
use futures::stream::FusedStream;
use futures::task::{Context, Poll};
use futures::{SinkExt, Stream, StreamExt};
use std::pin::Pin;

/// `Msg` represents a message which you can respond to.
pub trait Msg {
    type Resp;
}

/// Create an MPSC channel.
pub fn chan<M: Msg>(capacity: usize) -> (Sender<M>, Receiver<M>) {
    let (tx, rx) = mpsc::channel(capacity);
    let tx = Sender { tx };
    let rx = Receiver { rx };
    (tx, rx)
}

pub struct Sender<M: Msg> {
    tx: mpsc::Sender<(M, oneshot::Sender<M::Resp>)>,
}

impl<M: Msg> Sender<M> {
    pub async fn send(&mut self, msg: M) -> Option<M::Resp> {
        let (tx, rx) = oneshot::channel();
        self.tx.send((msg, tx)).await.ok()?;
        rx.await.ok()
    }
}

impl<M: Msg> Clone for Sender<M> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

pub struct Receiver<M: Msg> {
    rx: mpsc::Receiver<(M, oneshot::Sender<M::Resp>)>,
}

impl<M: Msg> Receiver<M> {
    pub async fn recv(&mut self) -> Option<MsgHandle<M>> {
        let (msg, tx) = self.rx.next().await?;
        Some(MsgHandle { msg, tx })
    }
}

impl<M: Msg> Stream for Receiver<M> {
    type Item = MsgHandle<M>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<MsgHandle<M>>> {
        match Pin::new(&mut self.rx).poll_next(cx) {
            Poll::Ready(Some((msg, tx))) => Poll::Ready(Some(MsgHandle { msg, tx })),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rx.size_hint()
    }
}

impl<M: Msg> FusedStream for Receiver<M> {
    fn is_terminated(&self) -> bool {
        self.rx.is_terminated()
    }
}

/// A message which you can respond to.
pub struct MsgHandle<M: Msg> {
    msg: M,
    tx: oneshot::Sender<M::Resp>,
}

impl<M: Msg> MsgHandle<M> {
    pub fn msg(&self) -> &M {
        &self.msg
    }

    pub fn into_msg(self) -> M {
        self.msg
    }

    /// Respond to the message. This will consume the original message.
    pub fn reply(self, msg: M::Resp) {
        let _ = self.tx.send(msg);
    }
}
