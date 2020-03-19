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
mod state;
mod utils;

use futures::channel::mpsc;
use futures::prelude::*;
use tokio::sync::broadcast;

use crate::common::{AuthToken, SessionId, UserId};

use messages::Message;
use state::State;

#[derive(Clone)]
pub struct StateActor {
    tx: mpsc::Sender<Message>,
}

impl StateActor {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(16);
        tokio::spawn(async move {
            let mut state = State::new();
            loop {
                match rx.next().await {
                    Some(msg) => state.handle(msg).await,
                    None => break,
                }
            }
        });
        Self { tx }
    }

    pub async fn connect(
        &mut self,
        session_id: SessionId,
        user_name: String,
    ) -> Option<(UserId, AuthToken)> {
        let (msg, rx) = Message::connect(session_id, user_name);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }

    pub async fn create(&mut self, user_name: String) -> Option<AuthToken> {
        let (msg, rx) = Message::create(user_name);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
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

    pub async fn garbage_collect(&mut self) {
        let _ = self.tx.send(Message::garbage_collect()).await;
    }

    pub async fn move_piece(&mut self, auth_token: AuthToken, change: String) -> Option<()> {
        let (msg, rx) = Message::move_piece(auth_token, change);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }

    pub async fn reconnect(
        &mut self,
        auth_token: AuthToken,
    ) -> Option<(SessionId, UserId, String)> {
        let (msg, rx) = Message::reconnect(auth_token);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
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

    pub async fn subscribe(
        &mut self,
        session_id: SessionId,
    ) -> Option<broadcast::Receiver<String>> {
        let (msg, rx) = Message::subscribe(session_id);
        self.tx.send(msg).await.ok()?;
        rx.await.ok()
    }
}
