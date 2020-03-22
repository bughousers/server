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
use futures::sink::SinkExt;
use futures::stream::StreamExt;

/// `Msg` represents a message which you can respond to.
pub trait Msg {
    type Msg;
    type Resp;
}

/// Create an MPSC channel.
pub fn channel<M: Msg>(capacity: usize) -> (Sender<M>, Receiver<M>) {
    let (tx, rx) = mpsc::channel(capacity);
    let tx = Sender { tx };
    let rx = Receiver { rx };
    (tx, rx)
}

#[derive(Clone)]
pub struct Sender<M: Msg> {
    tx: mpsc::Sender<(M::Msg, oneshot::Sender<M::Resp>)>,
}

impl<M: Msg> Sender<M> {
    pub async fn send(&mut self, msg: M::Msg) -> Option<M::Resp> {
        let (tx, rx) = oneshot::channel();
        self.tx.send((msg, tx)).await.ok()?;
        rx.await.ok()
    }
}

pub struct Receiver<M: Msg> {
    rx: mpsc::Receiver<(M::Msg, oneshot::Sender<M::Resp>)>,
}

impl<M: Msg> Receiver<M> {
    pub async fn recv(&mut self) -> Option<MsgHandle<M>> {
        let (msg, tx) = self.rx.next().await?;
        Some(MsgHandle { msg, tx })
    }
}

/// A message which you can respond to.
pub struct MsgHandle<M: Msg> {
    msg: M::Msg,
    tx: oneshot::Sender<M::Resp>,
}

impl<M: Msg> MsgHandle<M> {
    pub fn msg(&self) -> &M::Msg {
        &self.msg
    }

    pub fn into_msg(self) -> M::Msg {
        self.msg
    }

    /// Respond to the message. This will consume the original message.
    pub fn reply(self, msg: M::Resp) {
        let _ = self.tx.send(msg);
    }
}
