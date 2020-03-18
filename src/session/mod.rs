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

mod serialization;
mod utils;

use std::collections::HashMap;

use bughouse_rs::logic::ChessLogic;
use bughouse_rs::parse::parser::parse as parse_change;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::common::{AuthToken, User, UserId, UserStatus};

pub type Channel = mpsc::Sender<MsgContainer>;

pub struct Session {
    user_ids: HashMap<AuthToken, UserId>,
    users: HashMap<UserId, User>,
    owner: UserId,
    logic: ChessLogic,
    started: bool,
    tx: broadcast::Sender<String>,
}

impl Session {
    pub fn new(owner: UserId) -> Self {
        let (tx, _) = broadcast::channel(16);
        Self {
            user_ids: HashMap::with_capacity(4),
            users: HashMap::with_capacity(4),
            owner,
            logic: ChessLogic::new(),
            started: false,
            tx,
        }
    }

    pub async fn msg(ch: &mut Channel, msg: Msg) -> Option<Reply> {
        let (tx, mut rx) = mpsc::channel(1);
        ch.send(MsgContainer { msg: msg, tx }).await.ok()?;
        rx.recv().await
    }

    pub fn serve(mut self) -> Channel {
        let (tx, mut rx) = mpsc::channel(16);
        tokio::spawn(async move {
            loop {
                if let Some(msg_container) = rx.recv().await {
                    handle(&mut self, msg_container).await;
                } else {
                    break;
                }
            }
        });
        tx
    }

    fn notify_all(&mut self) -> Option<()> {
        let ev: serialization::Event = self.into();
        let ev = serde_json::to_string(&ev);
        let ev = format!("data: {}\n\n", ev.ok()?);
        self.tx.send(ev).ok()?;
        Some(())
    }
}

// Message types

pub struct MsgContainer {
    msg: Msg,
    tx: mpsc::Sender<Reply>,
}

pub enum Msg {
    Authorized {
        auth_token: AuthToken,
        msg: AuthorizedMsg,
    },
    DeployPiece {
        auth_token: AuthToken,
        piece: String,
        pos: String,
    },
    GetUser {
        user_id: UserId,
    },
    GetUserId {
        auth_token: AuthToken,
    },
    InsertUser {
        user_id: UserId,
        user: User,
    },
    InsertUserId {
        auth_token: AuthToken,
        user_id: UserId,
    },
    MovePiece {
        auth_token: AuthToken,
        change: String,
    },
    NotifyAll,
    Subscribe,
}

pub enum AuthorizedMsg {
    SetParticipants { user_ids: Vec<UserId> },
    Start,
}

pub enum Reply {
    Success,
    Failure,
    GetUser { user: User },
    GetUserId { user_id: UserId },
    Subscribe { rx: broadcast::Receiver<String> },
}

impl Reply {
    pub fn is_successful(&self) -> bool {
        match self {
            Reply::Failure => false,
            _ => true,
        }
    }
}

// Handlers

async fn handle(s: &mut Session, mut msg_container: MsgContainer) {
    let reply = match msg_container.msg {
        Msg::Authorized { auth_token, msg } => handle_authorized(s, auth_token, msg).await,
        Msg::DeployPiece {
            auth_token,
            piece,
            pos,
        } => handle_deploy_piece(s, auth_token, piece, pos).await,
        Msg::GetUser { user_id } => handle_get_user(s, user_id).await,
        Msg::GetUserId { auth_token } => handle_get_user_id(s, auth_token).await,
        Msg::InsertUser { user_id, user } => handle_insert_user(s, user_id, user).await,
        Msg::InsertUserId {
            auth_token,
            user_id,
        } => handle_insert_user_id(s, auth_token, user_id).await,
        Msg::MovePiece { auth_token, change } => handle_move_piece(s, auth_token, change).await,
        Msg::NotifyAll => handle_notify_all(s).await,
        Msg::Subscribe => handle_subscribe(s).await,
    };
    if let Some(reply) = reply {
        msg_container.tx.send(reply).await;
    } else {
        msg_container.tx.send(Reply::Failure).await;
    }
}

async fn handle_authorized(
    s: &mut Session,
    auth_token: AuthToken,
    msg: AuthorizedMsg,
) -> Option<Reply> {
    let user_id = s.user_ids.get(&auth_token)?;
    if *user_id == s.owner {
        match msg {
            AuthorizedMsg::SetParticipants { user_ids } => {
                handle_set_participants(s, user_ids).await
            }
            AuthorizedMsg::Start => handle_start(s).await,
        }
    } else {
        None
    }
}

async fn handle_deploy_piece(
    s: &mut Session,
    auth_token: AuthToken,
    piece: String,
    pos: String,
) -> Option<Reply> {
    let user_id = s.user_ids.get(&auth_token)?;
    let user = s.users.get(user_id)?;
    match user.status {
        UserStatus::Active {
            is_white,
            is_on_first_board,
        } => {
            let (col, row) = utils::parse_pos(&pos)?;
            if s.logic.deploy_piece(
                is_on_first_board,
                is_white,
                utils::parse_piece(&piece)?,
                row,
                col,
            ) {
                s.notify_all();
                Some(Reply::Success)
            } else {
                None
            }
        }
        _ => None,
    }
}

async fn handle_get_user(s: &mut Session, user_id: UserId) -> Option<Reply> {
    let user = s.users.get(&user_id)?.clone();
    Some(Reply::GetUser { user })
}

async fn handle_get_user_id(s: &mut Session, auth_token: AuthToken) -> Option<Reply> {
    let user_id = s.user_ids.get(&auth_token)?.clone();
    Some(Reply::GetUserId { user_id })
}

async fn handle_insert_user(s: &mut Session, user_id: UserId, user: User) -> Option<Reply> {
    if !utils::validate_user_name(&user.name) {
        return None;
    }
    s.users.insert(user_id, user);
    s.notify_all();
    Some(Reply::Success)
}

async fn handle_insert_user_id(
    s: &mut Session,
    auth_token: AuthToken,
    user_id: UserId,
) -> Option<Reply> {
    s.user_ids.insert(auth_token, user_id);
    Some(Reply::Success)
}

async fn handle_move_piece(
    s: &mut Session,
    auth_token: AuthToken,
    change: String,
) -> Option<Reply> {
    let user_id = s.user_ids.get(&auth_token)?;
    let user = s.users.get(user_id)?;
    match user.status {
        UserStatus::Active {
            is_white,
            is_on_first_board,
        } => {
            let is_whites_turn = if is_on_first_board {
                s.logic.white_active_1
            } else {
                s.logic.white_active_2
            };
            if is_whites_turn != is_white {
                return None;
            }
            let [i, j, i_new, j_new] = parse_change(&change);
            if s.logic.movemaker(is_on_first_board, i, j, i_new, j_new) {
                s.notify_all();
                Some(Reply::Success)
            } else {
                None
            }
        }
        _ => None,
    }
}

async fn handle_notify_all(s: &mut Session) -> Option<Reply> {
    s.notify_all()?;
    Some(Reply::Success)
}

async fn handle_subscribe(s: &mut Session) -> Option<Reply> {
    Some(Reply::Subscribe {
        rx: s.tx.subscribe(),
    })
}

async fn handle_set_participants(s: &mut Session, user_ids: Vec<UserId>) -> Option<Reply> {
    if s.started || user_ids.iter().any(|uid| !s.users.contains_key(uid)) {
        return None;
    }
    for uid in user_ids {
        s.users.get_mut(&uid)?.status = UserStatus::Inactive;
    }
    s.notify_all();
    Some(Reply::Success)
}

async fn handle_start(s: &mut Session) -> Option<Reply> {
    if s.started || s.users.values().filter(|&u| u.is_participant()).count() < 4 {
        return None;
    }
    s.started = true;
    s.notify_all();
    Some(Reply::Success)
}
