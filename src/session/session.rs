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

use bughouse_rs::logic::ChessLogic;
use tokio::sync::broadcast;

use crate::common::{AuthToken, User, UserId, UserStatus};

use super::messages::{Message, ResponseSender};
use super::serialization;
use super::utils;

pub struct Session {
    pub user_ids: HashMap<AuthToken, UserId>,
    pub users: HashMap<UserId, User>,
    pub owner_id: UserId,
    pub logic: ChessLogic,
    pub started: bool,
    pub tx: broadcast::Sender<String>,
    pub failed_broadcasts: usize,
}

impl Session {
    pub fn new(owner_id: UserId) -> Self {
        let (tx, _) = broadcast::channel(20);
        Self {
            user_ids: HashMap::with_capacity(4),
            users: HashMap::with_capacity(4),
            owner_id,
            logic: ChessLogic::new(),
            started: false,
            failed_broadcasts: 0,
            tx,
        }
    }

    async fn handle_deploy_piece(
        &mut self,
        auth_token: AuthToken,
        piece: String,
        pos: String,
        tx: ResponseSender<()>,
    ) -> Option<()> {
        if !self.started {
            return None;
        }
        let user_id = self.user_ids.get(&auth_token)?;
        let user = self.users.get(user_id)?;
        if let UserStatus::Active(b1, w) = user.status {
            let (col, row) = utils::parse_pos(&pos)?;
            let success = self
                .logic
                .deploy_piece(b1, w, utils::parse_piece(&piece)?, row, col);
            if success {
                self.handle_notify_all().await;
                let _ = tx.send(());
            }
        }
        Some(())
    }

    async fn handle_get_user(&self, user_id: UserId, tx: ResponseSender<User>) -> Option<()> {
        let user = self.users.get(&user_id)?;
        let _ = tx.send(user.clone());
        Some(())
    }

    async fn handle_get_user_id(
        &self,
        auth_token: AuthToken,
        tx: ResponseSender<UserId>,
    ) -> Option<()> {
        let user_id = self.user_ids.get(&auth_token)?;
        let _ = tx.send(user_id.clone());
        Some(())
    }

    async fn handle_insert_user(&mut self, user_id: UserId, user: User) -> Option<()> {
        if utils::validate_user_name(&user.name) {
            self.users.insert(user_id, user);
            self.handle_notify_all().await;
        }
        Some(())
    }

    async fn handle_insert_user_id(
        &mut self,
        auth_token: AuthToken,
        user_id: UserId,
    ) -> Option<()> {
        self.user_ids.insert(auth_token, user_id);
        Some(())
    }

    async fn handle_is_alive(&self, tx: ResponseSender<bool>) -> Option<()> {
        let _ = tx.send(self.failed_broadcasts < 20);
        Some(())
    }

    async fn handle_move_piece(
        &mut self,
        auth_token: AuthToken,
        change: String,
        tx: ResponseSender<()>,
    ) -> Option<()> {
        if !self.started {
            return None;
        }
        let user_id = self.user_ids.get(&auth_token)?;
        let user = self.users.get(user_id)?;
        if let UserStatus::Active(b1, w) = user.status {
            let is_whites_turn = if b1 {
                self.logic.white_active_1
            } else {
                self.logic.white_active_2
            };
            if is_whites_turn == w {
                let [i, j, i_new, j_new] = utils::parse_change(&change);
                if self.logic.movemaker(b1, i, j, i_new, j_new) {
                    self.handle_notify_all().await;
                    let _ = tx.send(());
                }
            }
        }
        Some(())
    }

    async fn handle_notify_all(&mut self) -> Option<()> {
        let ev: serialization::Event = self.into();
        let ev = serde_json::to_string(&ev).ok()?;
        let res = self.tx.send(format!("data: {}", ev));
        if res.is_ok() {
            self.failed_broadcasts = 0;
        }
        Some(())
    }

    async fn handle_set_participants(
        &mut self,
        auth_token: AuthToken,
        participants: Vec<UserId>,
        tx: ResponseSender<()>,
    ) -> Option<()> {
        let user_id = self.user_ids.get(&auth_token)?;
        if *user_id == self.owner_id
            && !self.started
            && participants.iter().all(|uid| self.users.contains_key(uid))
        {
            for uid in participants {
                self.users.get_mut(&uid)?.status = UserStatus::Inactive;
            }
            self.handle_notify_all().await;
            let _ = tx.send(());
        }
        Some(())
    }

    async fn handle_start(&mut self, auth_token: AuthToken, tx: ResponseSender<()>) -> Option<()> {
        let user_id = self.user_ids.get(&auth_token)?;
        if *user_id == self.owner_id
            && !self.started
            && self.users.values().filter(|&u| u.is_participant()).count() >= 4
        {
            self.started = true;
            self.handle_notify_all().await;
            let _ = tx.send(());
        }
        Some(())
    }

    async fn handle_subscribe(
        &mut self,
        tx: ResponseSender<broadcast::Receiver<String>>,
    ) -> Option<()> {
        let _ = tx.send(self.tx.subscribe());
        Some(())
    }

    pub async fn handle(&mut self, msg: Message) {
        match msg {
            Message::DeployPiece(tok, p, pos, tx) => {
                self.handle_deploy_piece(tok, p, pos, tx).await
            }
            Message::GetUser(uid, tx) => self.handle_get_user(uid, tx).await,
            Message::GetUserId(tok, tx) => self.handle_get_user_id(tok, tx).await,
            Message::InsertUser(uid, u) => self.handle_insert_user(uid, u).await,
            Message::InsertUserId(tok, uid) => self.handle_insert_user_id(tok, uid).await,
            Message::IsAlive(tx) => self.handle_is_alive(tx).await,
            Message::MovePiece(tok, c, tx) => self.handle_move_piece(tok, c, tx).await,
            Message::NotifyAll => self.handle_notify_all().await,
            Message::SetParticipants(tok, p, tx) => self.handle_set_participants(tok, p, tx).await,
            Message::Start(tok, tx) => self.handle_start(tok, tx).await,
            Message::Subscribe(tx) => self.handle_subscribe(tx).await,
        };
    }
}
