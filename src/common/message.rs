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

use std::{any::Any, ops::Deref};
use tokio::sync::{mpsc, oneshot};

pub trait Message: Any + Send + 'static {
    type Response: Any + Send + 'static;
}

pub fn channel(capacity: usize) -> (Tx, Rx) {
    let (tx, rx) = mpsc::channel(capacity);
    (Tx { tx }, Rx { rx })
}

pub struct Tx {
    tx: mpsc::Sender<(
        Box<dyn Any + Send + 'static>,
        oneshot::Sender<Box<dyn Any + Send + 'static>>,
    )>,
}

impl Tx {
    pub async fn send<M: Message>(&mut self, msg: M) -> Option<Box<M::Response>> {
        let msg = Box::new(msg);
        let (tx, rx) = oneshot::channel();
        self.tx.send((msg, tx)).await.ok()?;
        rx.await.ok()?.downcast::<M::Response>().ok()
    }
}

pub struct Rx {
    rx: mpsc::Receiver<(
        Box<dyn Any + Send + 'static>,
        oneshot::Sender<Box<dyn Any + Send + 'static>>,
    )>,
}

impl Rx {
    pub async fn recv(&mut self) -> Option<Envelope> {
        let (msg, tx) = self.rx.recv().await?;
        Some(Envelope { msg, tx })
    }
}

pub struct Envelope {
    msg: Box<dyn Any + Send + 'static>,
    tx: oneshot::Sender<Box<dyn Any + Send + 'static>>,
}

impl Envelope {
    pub fn open<M: Message>(&self) -> Option<EnvelopeContent<M>> {
        let msg = self.msg.downcast_ref::<M>()?;
        Some(EnvelopeContent { msg })
    }

    pub fn reply(self, msg: Box<dyn Any + Send + 'static>) {
        let _ = self.tx.send(msg);
    }

    pub fn reply_to<M: Message>(self, _: EnvelopeContent<M>, msg: M::Response) {
        self.reply(Box::new(msg));
    }
}

pub struct EnvelopeContent<'a, M: Message> {
    msg: &'a M,
}

impl<M: Message> Deref for EnvelopeContent<'_, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        self.msg
    }
}
