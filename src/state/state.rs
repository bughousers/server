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

use tokio::spawn;
use tokio::sync::mpsc::*;

use crate::state::misc::{AuthToken, SessionId, UserId};
use crate::state::session::Session;

pub struct State {
    sessions: HashMap<SessionId, Session>,
    session_ids: HashMap<UserId, SessionId>,
    auth_tokens: HashMap<UserId, AuthToken>,
}

impl State {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            session_ids: HashMap::new(),
            auth_tokens: HashMap::new(),
        }
    }

    pub fn serve(mut self) -> Channel {
        let (tx, mut rx) = unbounded_channel::<Msg>();
        spawn(async move {
            loop {
                let msg = rx.recv().await;
                if let Some(msg) = msg {
                    handle(&mut self, msg).await;
                }
            }
        });
        tx
    }
}

pub type Channel = UnboundedSender<Msg>;
pub type RespChannel = Sender<MsgResp>;

pub struct Msg {
    pub data: MsgData,
    pub resp_channel: RespChannel,
}

pub enum MsgData {
    Connect(SessionId, String),
    Create(String),
    Reconnect(UserId, AuthToken),
}

pub enum MsgResp {
    Connected(UserId, AuthToken),
    ConnectFailure,
    Created(SessionId, UserId, AuthToken),
    Reconnected(SessionId, String),
    ReconnectFailure,
}

async fn handle(state: &mut State, msg: Msg) -> Option<()> {
    match msg.data {
        MsgData::Connect(sid, n) => handle_connect(state, msg.resp_channel, sid, n).await,
        MsgData::Create(n) => handle_create(state, msg.resp_channel, n).await,
        MsgData::Reconnect(uid, tok) => handle_reconnect(state, msg.resp_channel, uid, tok).await,
    }
}

// Handle Connect messages

async fn handle_connect(
    state: &mut State,
    mut ch: RespChannel,
    session_id: SessionId,
    user_name: String,
) -> Option<()> {
    let resp = if let Some(session) = state.sessions.get_mut(&session_id) {
        let user_id = UserId::new();
        let auth_token = AuthToken::new();
        session.user_names.insert(user_id.clone(), user_name);
        state.session_ids.insert(user_id.clone(), session_id);
        state
            .auth_tokens
            .insert(user_id.clone(), auth_token.clone());
        MsgResp::Connected(user_id, auth_token)
    } else {
        MsgResp::ConnectFailure
    };
    ch.send(resp).await;
    Some(())
}

// Handle Create messages

async fn handle_create(state: &mut State, mut ch: RespChannel, user_name: String) -> Option<()> {
    let session_id = SessionId::new();
    let user_id = UserId::new();
    let auth_token = AuthToken::new();
    let mut session = Session::new(user_id.clone());
    session.user_names.insert(user_id.clone(), user_name);
    state.sessions.insert(session_id.clone(), session);
    state
        .session_ids
        .insert(user_id.clone(), session_id.clone());
    state
        .auth_tokens
        .insert(user_id.clone(), auth_token.clone());
    ch.send(MsgResp::Created(session_id, user_id, auth_token))
        .await;
    Some(())
}

// Handle Reconnect messages

async fn handle_reconnect(
    state: &mut State,
    mut ch: RespChannel,
    user_id: UserId,
    auth_token: AuthToken,
) -> Option<()> {
    let msg_resp = if let Some(sid) = auth(state, &user_id, &auth_token) {
        MsgResp::Reconnected(
            sid.clone(),
            state.sessions.get(sid)?.user_names.get(&user_id)?.into(),
        )
    } else {
        MsgResp::ReconnectFailure
    };
    ch.send(msg_resp).await;
    Some(())
}

// Helper functions

fn auth<'a>(state: &'a State, user_id: &UserId, auth_token: &AuthToken) -> Option<&'a SessionId> {
    let tok = state.auth_tokens.get(user_id)?;
    if tok == auth_token {
        state.session_ids.get(user_id)
    } else {
        None
    }
}
