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

use rand::Rng;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::common::{AuthToken, SessionId, User, UserId};
use crate::session;
use crate::session::Session;

pub type Channel = mpsc::Sender<MsgContainer>;

pub struct State {
    sessions: HashMap<SessionId, session::Channel>,
    session_ids: HashMap<AuthToken, SessionId>,
}

impl State {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            session_ids: HashMap::new(),
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
}

// Message types

pub struct MsgContainer {
    msg: Msg,
    tx: mpsc::Sender<Reply>,
}

pub enum Msg {
    Authenticated {
        auth_token: AuthToken,
        msg: AuthenticatedMsg,
    },
    Connect {
        session_id: SessionId,
        user_name: String,
    },
    Create {
        user_name: String,
    },
    Subscribe {
        session_id: SessionId,
    },
}

pub enum AuthenticatedMsg {
    Reconnect,
    Relay { msg: session::Msg },
}

pub enum Reply {
    Success,
    Failure,
    Connect {
        user_id: UserId,
        auth_token: AuthToken,
    },
    Create {
        auth_token: AuthToken,
    },
    Reconnect {
        session_id: SessionId,
        user_id: UserId,
        user_name: String,
    },
    Relay {
        reply: session::Reply,
    },
    Subscribe {
        rx: broadcast::Receiver<String>,
    },
}

// Handlers

async fn handle(s: &mut State, mut msg_container: MsgContainer) {
    let reply = match msg_container.msg {
        Msg::Authenticated { auth_token, msg } => handle_authenticated(s, auth_token, msg).await,
        Msg::Connect {
            session_id,
            user_name,
        } => handle_connect(s, session_id, user_name).await,
        Msg::Create { user_name } => handle_create(s, user_name).await,
        Msg::Subscribe { session_id } => handle_subscribe(s, session_id).await,
    };
    if let Some(reply) = reply {
        msg_container.tx.send(reply).await;
    } else {
        msg_container.tx.send(Reply::Failure).await;
    }
}

async fn handle_authenticated(
    s: &mut State,
    auth_token: AuthToken,
    msg: AuthenticatedMsg,
) -> Option<Reply> {
    let sid = s.session_ids.get(&auth_token)?.clone();
    match msg {
        AuthenticatedMsg::Reconnect => handle_reconnect(s, sid, auth_token).await,
        AuthenticatedMsg::Relay { msg } => handle_relay(s, sid, msg).await,
    }
}

async fn handle_connect(s: &mut State, session_id: SessionId, user_name: String) -> Option<Reply> {
    let user_id = rand_user_id();
    let auth_token = rand_auth_token();
    let ch = s.sessions.get_mut(&session_id)?;

    if !Session::msg(
        ch,
        session::Msg::InsertUser {
            user_id: user_id.clone(),
            user: User::new(user_name),
        },
    )
    .await?
    .is_successful()
    {
        return None;
    }

    if !Session::msg(
        ch,
        session::Msg::InsertUserId {
            auth_token: auth_token.clone(),
            user_id: user_id.clone(),
        },
    )
    .await?
    .is_successful()
    {
        return None;
    }

    s.session_ids.insert(auth_token.clone(), session_id);
    Some(Reply::Connect {
        user_id,
        auth_token,
    })
}

async fn handle_create(s: &mut State, user_name: String) -> Option<Reply> {
    let session_id = rand_session_id();
    let user_id = rand_user_id();
    let auth_token = rand_auth_token();
    let mut ch = Session::new(user_id.clone()).serve();

    if !Session::msg(
        &mut ch,
        session::Msg::InsertUser {
            user_id: user_id.clone(),
            user: User::new(user_name),
        },
    )
    .await?
    .is_successful()
    {
        return None;
    }

    if !Session::msg(
        &mut ch,
        session::Msg::InsertUserId {
            auth_token: auth_token.clone(),
            user_id,
        },
    )
    .await?
    .is_successful()
    {
        return None;
    }

    s.sessions.insert(session_id.clone(), ch);
    s.session_ids.insert(auth_token.clone(), session_id);
    Some(Reply::Create { auth_token })
}

async fn handle_subscribe(s: &mut State, session_id: SessionId) -> Option<Reply> {
    let ch = s.sessions.get_mut(&session_id)?;
    let reply = Session::msg(ch, session::Msg::Subscribe).await?;
    match reply {
        session::Reply::Subscribe { rx } => Some(Reply::Subscribe { rx }),
        _ => None,
    }
}

async fn handle_reconnect(
    s: &mut State,
    session_id: SessionId,
    auth_token: AuthToken,
) -> Option<Reply> {
    let ch = s.sessions.get_mut(&session_id)?;
    let reply = Session::msg(ch, session::Msg::GetUserId { auth_token }).await?;
    if let session::Reply::GetUserId { user_id } = reply {
        let reply = Session::msg(
            ch,
            session::Msg::GetUser {
                user_id: user_id.clone(),
            },
        )
        .await?;
        if let session::Reply::GetUser { user } = reply {
            Some(Reply::Reconnect {
                session_id,
                user_id,
                user_name: user.name,
            })
        } else {
            None
        }
    } else {
        None
    }
}

async fn handle_relay(s: &mut State, session_id: SessionId, msg: session::Msg) -> Option<Reply> {
    let ch = s.sessions.get_mut(&session_id)?;
    let reply = Session::msg(ch, msg).await?;
    match reply {
        session::Reply::Success => Some(Reply::Success),
        reply => Some(Reply::Relay { reply }),
    }
}

// Helper functions

fn rand_auth_token() -> AuthToken {
    rand_alphanum_string(32)
}

fn rand_user_id() -> UserId {
    rand_alphanum_string(16)
}

fn rand_session_id() -> SessionId {
    rand_alphanum_string(4)
}

fn rand_alphanum_string(len: usize) -> String {
    std::iter::repeat(())
        .map(|()| rand::thread_rng().sample(rand::distributions::Alphanumeric))
        .take(len)
        .collect()
}
