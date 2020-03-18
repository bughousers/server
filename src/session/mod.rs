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

mod messages;
mod serialization;
mod session;
mod utils;

use futures::channel::mpsc;
use futures::prelude::*;
use tokio::sync::broadcast;

use crate::common::{AuthToken, User, UserId};

use messages::Message;
use session::Session;

pub struct SessionActor {
    tx: mpsc::Sender<Message>,
}

impl SessionActor {
    pub fn new(owner_id: String) -> Self {
        let (tx, mut rx) = mpsc::channel(16);
        tokio::spawn(async move {
            let mut session = Session::new(owner_id);
            loop {
                match rx.next().await {
                    Some(msg) => session.handle(msg).await,
                    None => break,
                }
            }
        });
        Self { tx }
    }

    pub async fn deploy_piece(
        &mut self,
        auth_token: AuthToken,
        piece: String,
        pos: String,
    ) -> Option<()> {
        let (msg, rx) = Message::deploy_piece(auth_token, piece, pos);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }

    pub async fn get_user(&mut self, user_id: UserId) -> Option<User> {
        let (msg, rx) = Message::get_user(user_id);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }

    pub async fn get_user_id(&mut self, auth_token: AuthToken) -> Option<UserId> {
        let (msg, rx) = Message::get_user_id(auth_token);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }

    pub async fn insert_user(&mut self, user_id: UserId, user: User) {
        let _ = self.tx.send(Message::insert_user(user_id, user)).await;
    }

    pub async fn insert_user_id(&mut self, auth_token: AuthToken, user_id: UserId) {
        let _ = self
            .tx
            .send(Message::insert_user_id(auth_token, user_id))
            .await;
    }

    pub async fn is_alive(&mut self) -> Option<bool> {
        let (msg, rx) = Message::is_alive();
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }

    pub async fn move_piece(&mut self, auth_token: AuthToken, change: String) -> Option<()> {
        let (msg, rx) = Message::move_piece(auth_token, change);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }

    pub async fn notify_all(&mut self) {
        let _ = self.tx.send(Message::notify_all()).await;
    }

    pub async fn set_participants(
        &mut self,
        auth_token: AuthToken,
        participants: Vec<UserId>,
    ) -> Option<()> {
        let (msg, rx) = Message::set_participants(auth_token, participants);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }

    pub async fn start(&mut self, auth_token: AuthToken) -> Option<()> {
        let (msg, rx) = Message::start(auth_token);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }

    pub async fn subscribe(&mut self) -> Option<broadcast::Receiver<String>> {
        let (msg, rx) = Message::subscribe();
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }
}
