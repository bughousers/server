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

use std::collections::HashMap;

use tokio::sync::broadcast;

use crate::common::{AuthToken, SessionId, User, UserId};
use crate::session::SessionActor;

use super::messages::{Message, ResponseSender};
use super::utils;

pub struct State {
    pub sessions: HashMap<SessionId, SessionActor>,
    pub session_ids: HashMap<AuthToken, SessionId>,
}

impl State {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            session_ids: HashMap::new(),
        }
    }

    async fn handle_connect(
        &mut self,
        session_id: SessionId,
        user_name: String,
        tx: ResponseSender<(UserId, AuthToken)>,
    ) -> Option<()> {
        let user_id = utils::rand_user_id();
        let auth_token = utils::rand_auth_token();
        let session = self.sessions.get_mut(&session_id)?;
        session
            .insert_user(user_id.clone(), User::new(user_name))
            .await;
        session
            .insert_user_id(auth_token.clone(), user_id.clone())
            .await;
        self.session_ids.insert(auth_token.clone(), session_id);
        let _ = tx.send((user_id, auth_token));
        Some(())
    }

    async fn handle_create(
        &mut self,
        user_name: String,
        tx: ResponseSender<AuthToken>,
    ) -> Option<()> {
        let session_id = utils::rand_session_id();
        let user_id = utils::rand_user_id();
        let auth_token = utils::rand_auth_token();
        let mut session = SessionActor::new(user_id.clone());
        session
            .insert_user(user_id.clone(), User::new(user_name))
            .await;
        session.insert_user_id(auth_token.clone(), user_id).await;
        self.sessions.insert(session_id.clone(), session);
        self.session_ids.insert(auth_token.clone(), session_id);
        let _ = tx.send(auth_token);
        Some(())
    }

    async fn handle_deploy_piece(
        &mut self,
        auth_token: AuthToken,
        piece: String,
        pos: String,
        tx: ResponseSender<()>,
    ) -> Option<()> {
        let session_id = self.session_ids.get(&auth_token)?;
        let session = self.sessions.get_mut(session_id)?;
        if session.deploy_piece(auth_token, piece, pos).await.is_some() {
            let _ = tx.send(());
        }
        Some(())
    }

    async fn handle_garbage_collect(&mut self) -> Option<()> {
        // Mark
        let mut marked = Vec::new();
        for (sid, s) in self.sessions.iter_mut() {
            let is_alive = s.is_alive().await;
            if is_alive.unwrap_or(false) {
                marked.push(sid.to_owned());
            }
        }
        // Sweep
        self.session_ids = self
            .session_ids
            .iter()
            .filter(|(_, sid)| !marked.contains(sid))
            .map(|(tok, sid)| (tok.to_owned(), sid.to_owned()))
            .collect();
        for sid in marked {
            self.sessions.remove(&sid);
        }
        Some(())
    }

    async fn handle_move_piece(
        &mut self,
        auth_token: AuthToken,
        change: String,
        tx: ResponseSender<()>,
    ) -> Option<()> {
        let session_id = self.session_ids.get(&auth_token)?;
        let session = self.sessions.get_mut(session_id)?;
        if session.move_piece(auth_token, change).await.is_some() {
            let _ = tx.send(());
        }
        Some(())
    }

    async fn handle_reconnect(
        &mut self,
        auth_token: AuthToken,
        tx: ResponseSender<(SessionId, UserId, String)>,
    ) -> Option<()> {
        let session_id = self.session_ids.get(&auth_token)?;
        let session = self.sessions.get_mut(session_id)?;
        let user_id = session.get_user_id(auth_token).await?;
        let user = session.get_user(user_id.clone()).await?;
        let _ = tx.send((session_id.to_owned(), user_id, user.name));
        Some(())
    }

    async fn handle_set_participants(
        &mut self,
        auth_token: AuthToken,
        participants: Vec<UserId>,
        tx: ResponseSender<()>,
    ) -> Option<()> {
        let session_id = self.session_ids.get(&auth_token)?;
        let session = self.sessions.get_mut(session_id)?;
        session.set_participants(auth_token, participants).await?;
        let _ = tx.send(());
        Some(())
    }

    async fn handle_start(&mut self, auth_token: AuthToken, tx: ResponseSender<()>) -> Option<()> {
        let session_id = self.session_ids.get(&auth_token)?;
        let session = self.sessions.get_mut(session_id)?;
        session.start(auth_token).await?;
        let _ = tx.send(());
        Some(())
    }

    async fn handle_subscribe(
        &mut self,
        session_id: SessionId,
        tx: ResponseSender<broadcast::Receiver<String>>,
    ) -> Option<()> {
        let session = self.sessions.get_mut(&session_id)?;
        let rx = session.subscribe().await?;
        let _ = tx.send(rx);
        Some(())
    }

    pub async fn handle(&mut self, msg: Message) {
        match msg {
            Message::Connect(sid, n, tx) => self.handle_connect(sid, n, tx).await,
            Message::Create(n, tx) => self.handle_create(n, tx).await,
            Message::DeployPiece(tok, p, pos, tx) => {
                self.handle_deploy_piece(tok, p, pos, tx).await
            }
            Message::GarbageCollect => self.handle_garbage_collect().await,
            Message::MovePiece(tok, c, tx) => self.handle_move_piece(tok, c, tx).await,
            Message::Reconnect(tok, tx) => self.handle_reconnect(tok, tx).await,
            Message::SetParticipants(tok, p, tx) => self.handle_set_participants(tok, p, tx).await,
            Message::Start(tok, tx) => self.handle_start(tok, tx).await,
            Message::Subscribe(tok, tx) => self.handle_subscribe(tok, tx).await,
        };
    }
}
