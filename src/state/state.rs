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
    Create(String),
    Reconnect(SessionId, UserId, AuthToken),
}

pub enum MsgResp {
    Created(SessionId, UserId, AuthToken),
    Reconnected(String),
    ReconnectFailure,
}

async fn handle(state: &mut State, msg: Msg) -> Option<()> {
    match msg.data {
        MsgData::Create(n) => handle_create(state, msg.resp_channel, n).await,
        MsgData::Reconnect(sid, uid, tok) => {
            handle_reconnect(state, msg.resp_channel, sid, uid, tok).await
        }
    }
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
    session_id: SessionId,
    user_id: UserId,
    auth_token: AuthToken,
) -> Option<()> {
    let msg_resp = if auth(state, &session_id, &user_id, &auth_token) {
        MsgResp::Reconnected(
            state
                .sessions
                .get(&session_id)?
                .user_names
                .get(&user_id)?
                .into(),
        )
    } else {
        MsgResp::ReconnectFailure
    };
    ch.send(msg_resp).await;
    Some(())
}

// Helper functions

fn auth(state: &State, session_id: &SessionId, user_id: &UserId, auth_token: &AuthToken) -> bool {
    let sid = state.session_ids.get(user_id);
    let tok = state.auth_tokens.get(user_id);
    if let (Some(sid), Some(tok)) = (sid, tok) {
        sid == session_id && tok == auth_token
    } else {
        false
    }
}
