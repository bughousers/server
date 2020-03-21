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

#![allow(unused_must_use)]

mod utils;

use crate::common::*;
use crate::dispatcher::Message as DispatcherMessage;
use crate::dispatcher::MessageError as DispatcherMessageError;
use crate::registry::Message as RegistryMessage;
use bughouse_rs::infoCourier::infoCourier::gen_yfen;
use bughouse_rs::logic::ChessLogic;
use std::collections::HashMap;
use tokio::sync::oneshot::Sender;
use tokio::sync::{broadcast, mpsc};

const BROADCAST_CHANNEL_CAPACITY: usize = 10;
const CHANNEL_CAPACITY: usize = 5;
const MAP_START_CAPACITY: usize = 4;
const MAX_NUM_OF_FAILED_BROADCASTS: usize = 20;
const MAX_NUM_OF_PARTICIPANTS: usize = 5;
const MAX_NUM_OF_USERS: usize = std::u8::MAX as usize + 1;

#[derive(Debug)]
pub enum Message {
    Request(Request, Sender<DispatcherMessage>),
    Subscribe(Sender<broadcast::Receiver<String>>),
}

pub struct Session {
    id: SessionId,
    parent_handle: mpsc::Sender<RegistryMessage>,
    users: HashMap<UserId, User>,
    user_ids: HashMap<AuthToken, UserId>,
    logic: ChessLogic,
    started: bool,
    broadcast_tx: broadcast::Sender<String>,
    failed_broadcasts: usize,
}

impl Session {
    pub fn spawn(
        id: SessionId,
        parent_handle: mpsc::Sender<RegistryMessage>,
    ) -> mpsc::Sender<Message> {
        let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        let session = Self {
            id,
            parent_handle,
            users: HashMap::with_capacity(MAP_START_CAPACITY),
            user_ids: HashMap::with_capacity(MAP_START_CAPACITY),
            logic: ChessLogic::new(),
            started: false,
            broadcast_tx,
            failed_broadcasts: 0,
        };
        tokio::spawn(async move {
            let (mut session, mut rx) = (session, rx);
            while let Some(msg) = rx.recv().await {
                handle(&mut session, msg).await;
            }
        });
        tx
    }

    async fn notify_all(&mut self) {
        let json = serde_json::to_string(&Event {
            users: self.users.values().map(|u| u.clone()).collect(),
            board: gen_yfen(&self.logic),
            started: self.started,
        })
        .expect("Serialization failed");
        // Keep in mind that a broadcast only fails when **nobody** receives the
        // message.
        if self.broadcast_tx.send(json).is_ok() {
            self.failed_broadcasts = 0;
        } else {
            self.failed_broadcasts += 1;
        }
        // Clearly, no one is listening. The session can be deleted.
        if self.failed_broadcasts >= MAX_NUM_OF_FAILED_BROADCASTS {
            self.parent_handle
                .send(RegistryMessage::Deregister(self.id.clone()))
                .await
                .expect("Registry channel is closed");
        }
    }
}

async fn handle(s: &mut Session, msg: Message) {
    match msg {
        Message::Request(req, tx) => handle_request(s, req, tx).await,
        Message::Subscribe(tx) => {
            tx.send(s.broadcast_tx.subscribe());
        }
    }
}

async fn handle_request(s: &mut Session, req: Request, tx: Sender<DispatcherMessage>) {
    match req {
        Request::Authenticated { auth_token, data } => {
            handle_authenticated(s, auth_token, data, tx).await
        }
        Request::Connect {
            session_id,
            user_name,
        } => handle_connect(s, session_id, user_name, tx).await,
        Request::Create { user_name } => handle_create(s, user_name, tx).await,
    }
}

async fn handle_authenticated(
    s: &mut Session,
    auth_token: AuthToken,
    data: Authenticated,
    tx: Sender<DispatcherMessage>,
) {
    if let Some(user_id) = s.user_ids.get(&auth_token) {
        // We have to dereference `user_id` first, otherwise the borrow checker
        // won't let us borrow `s`.
        let user_id = *user_id;
        match data {
            Authenticated::DeployPiece { piece, pos } => {
                handle_deploy_piece(s, user_id, piece, pos, tx).await
            }
            Authenticated::MovePiece { change } => handle_move_piece(s, user_id, change, tx).await,
            Authenticated::Reconnect => handle_reconnect(s, user_id, tx).await,
            Authenticated::SetParticipants { participants } => {
                handle_set_participants(s, user_id, participants, tx).await
            }
            Authenticated::Start => handle_start(s, user_id, tx).await,
        }
    } else {
        tx.send(DispatcherMessage::Error(
            DispatcherMessageError::AuthTokenInvalid,
        ));
    }
}

async fn handle_connect(
    s: &mut Session,
    session_id: SessionId,
    user_name: String,
    tx: Sender<DispatcherMessage>,
) {
    let user_id = s.user_ids.len();
    if user_id >= MAX_NUM_OF_USERS {
        tx.send(DispatcherMessage::Error(
            DispatcherMessageError::TooManyUsers,
        ));
        return;
    }
    let user_id = UserId::new(user_id as u8);
    let user = User::new(user_id, user_name);
    let auth_token = AuthToken::new();
    s.users.insert(user_id, user.clone());
    s.user_ids.insert(auth_token.clone(), user_id);
    s.parent_handle
        .send(RegistryMessage::Register(auth_token.clone(), session_id))
        .await
        .expect("Registry channel is closed");
    tx.send(DispatcherMessage::Response(Response::Connected {
        user,
        auth_token,
    }));
    s.notify_all().await;
}

async fn handle_create(s: &mut Session, user_name: String, tx: Sender<DispatcherMessage>) {
    let user_id = UserId::new(0);
    let user = User::new(user_id, user_name);
    let auth_token = AuthToken::new();
    s.users.insert(user_id, user);
    s.user_ids.insert(auth_token.clone(), user_id);
    s.parent_handle
        .send(RegistryMessage::Register(auth_token.clone(), s.id.clone()))
        .await
        .expect("Registry channel is closed");
    tx.send(DispatcherMessage::Response(Response::Created {
        auth_token,
    }));
}

async fn handle_deploy_piece(
    s: &mut Session,
    user_id: UserId,
    piece: String,
    pos: String,
    tx: Sender<DispatcherMessage>,
) {
    if !s.started {
        tx.send(DispatcherMessage::Error(
            DispatcherMessageError::PreconditionFailure,
        ));
        return;
    }
    let user = s
        .users
        .get(&user_id)
        .expect("User ID is not associated with any user");
    if let UserStatus::Active(b1, w) = user.status {
        let pos = utils::parse_pos(&pos);
        let piece = utils::parse_piece(&piece);
        if let (Some((col, row)), Some(piece)) = (pos, piece) {
            if s.logic.deploy_piece(b1, w, piece, row, col) {
                tx.send(DispatcherMessage::Response(Response::Success));
                s.notify_all().await;
            } else {
                tx.send(DispatcherMessage::Error(
                    DispatcherMessageError::PreconditionFailure,
                ));
            }
        } else {
            tx.send(DispatcherMessage::Error(
                DispatcherMessageError::CannotParse,
            ));
        }
    } else {
        tx.send(DispatcherMessage::Error(
            DispatcherMessageError::PreconditionFailure,
        ));
    }
}

async fn handle_move_piece(
    s: &mut Session,
    user_id: UserId,
    change: String,
    tx: Sender<DispatcherMessage>,
) {
    if !s.started {
        tx.send(DispatcherMessage::Error(
            DispatcherMessageError::PreconditionFailure,
        ));
        return;
    }
    let user = s
        .users
        .get(&user_id)
        .expect("User ID is not associated with any user");
    if let UserStatus::Active(b1, w) = user.status {
        let is_whites_turn = if b1 {
            s.logic.white_active_1
        } else {
            s.logic.white_active_2
        };
        if is_whites_turn == w {
            let [i, j, i_new, j_new] = utils::parse_change(&change);
            if s.logic.movemaker(b1, i, j, i_new, j_new) {
                tx.send(DispatcherMessage::Response(Response::Success));
                s.notify_all().await;
            } else {
                tx.send(DispatcherMessage::Error(
                    DispatcherMessageError::PreconditionFailure,
                ));
            }
        }
    } else {
        tx.send(DispatcherMessage::Error(
            DispatcherMessageError::PreconditionFailure,
        ));
    }
}

async fn handle_reconnect(s: &mut Session, user_id: UserId, tx: Sender<DispatcherMessage>) {
    let user = s
        .users
        .get(&user_id)
        .expect("User ID is not associated with any user")
        .clone();
    tx.send(DispatcherMessage::Response(Response::Reconnected {
        session_id: s.id.clone(),
        user,
    }));
}

async fn handle_set_participants(
    s: &mut Session,
    user_id: UserId,
    participants: Vec<UserId>,
    tx: Sender<DispatcherMessage>,
) {
    if user_id != UserId::OWNER {
        tx.send(DispatcherMessage::Error(
            DispatcherMessageError::MustBeSessionOwner,
        ));
        return;
    } else if participants.iter().any(|uid| !s.users.contains_key(uid)) {
        tx.send(DispatcherMessage::Error(
            DispatcherMessageError::PreconditionFailure,
        ));
        return;
    }
    for uid in participants {
        s.users.get_mut(&uid).unwrap().status = UserStatus::Inactive;
    }
    tx.send(DispatcherMessage::Response(Response::Success));
    s.notify_all().await;
}

async fn handle_start(s: &mut Session, user_id: UserId, tx: Sender<DispatcherMessage>) {
    if user_id != UserId::OWNER {
        tx.send(DispatcherMessage::Error(
            DispatcherMessageError::MustBeSessionOwner,
        ));
        return;
    } else if !(4..=MAX_NUM_OF_PARTICIPANTS)
        .contains(&s.users.values().filter(|&u| u.is_participant()).count())
    {
        tx.send(DispatcherMessage::Error(
            DispatcherMessageError::PreconditionFailure,
        ));
        return;
    }
    s.started = true;
    tx.send(DispatcherMessage::Response(Response::Success));
    s.notify_all().await;
}
