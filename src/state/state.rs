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

use super::misc::{AuthToken, SessionId, UserId};
use super::session::Session;

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
    Authenticate(UserId, AuthToken),
    ChangeParticipants(SessionId, UserId, Vec<UserId>),
    Connect(SessionId, String),
    Create(String),
    Deploy(SessionId, UserId, String, String),
    Move(SessionId, UserId, String),
    Reconnect(SessionId, UserId),
    Start(SessionId, UserId),
}

pub enum MsgResp {
    Authenticated(SessionId),
    AuthenticateFailure,
    ChangedParticipants,
    ChangeParticipantsFailure,
    Connected(UserId, AuthToken),
    ConnectFailure,
    Created(SessionId, UserId, AuthToken),
    Deployed,
    DeployFailure,
    Moved,
    MoveFailure,
    Reconnected(SessionId, String),
    ReconnectFailure,
    Started,
    StartFailure,
}

async fn handle(state: &mut State, msg: Msg) -> Option<()> {
    match msg.data {
        MsgData::Authenticate(uid, tok) => {
            handle_authenticate(state, msg.resp_channel, uid, tok).await
        }
        MsgData::ChangeParticipants(sid, uid, p) => {
            handle_change_participants(state, msg.resp_channel, sid, uid, p).await
        }
        MsgData::Connect(sid, n) => handle_connect(state, msg.resp_channel, sid, n).await,
        MsgData::Create(n) => handle_create(state, msg.resp_channel, n).await,
        MsgData::Deploy(sid, uid, p, pos) => {
            handle_deploy(state, msg.resp_channel, sid, uid, p, pos).await
        }
        MsgData::Move(sid, uid, c) => handle_move(state, msg.resp_channel, sid, uid, c).await,
        MsgData::Reconnect(sid, uid) => handle_reconnect(state, msg.resp_channel, sid, uid).await,
        MsgData::Start(sid, uid) => handle_start(state, msg.resp_channel, sid, uid).await,
    }
}

// Handle Authenticate messages

async fn handle_authenticate(
    state: &mut State,
    mut ch: RespChannel,
    user_id: UserId,
    auth_token: AuthToken,
) -> Option<()> {
    let msg_resp = if let Some(sid) = auth(state, &user_id, &auth_token) {
        MsgResp::Authenticated(sid)
    } else {
        MsgResp::AuthenticateFailure
    };
    ch.send(msg_resp).await;
    Some(())
}

// Handle ChangeParticipants messages

async fn handle_change_participants(
    state: &mut State,
    mut ch: RespChannel,
    session_id: SessionId,
    user_id: UserId,
    participants: Vec<UserId>,
) -> Option<()> {
    let session = state.sessions.get_mut(&session_id)?;
    let msg_resp = if session.set_participants(&user_id, participants) {
        MsgResp::ChangedParticipants
    } else {
        MsgResp::ChangeParticipantsFailure
    };
    ch.send(msg_resp).await;
    Some(())
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
        session.set_user_name(user_id.clone(), user_name);
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
    session.set_user_name(user_id.clone(), user_name);
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

// Handle Deploy messages

async fn handle_deploy(
    state: &mut State,
    mut ch: RespChannel,
    session_id: SessionId,
    user_id: UserId,
    piece: String,
    pos: String,
) -> Option<()> {
    let msg_resp = {
        let session = state.sessions.get_mut(&session_id)?;
        if session.deploy_piece(&user_id, piece, pos).is_some() {
            MsgResp::Deployed
        } else {
            MsgResp::DeployFailure
        }
    };
    ch.send(msg_resp).await;
    Some(())
}

// Handle Move messages

async fn handle_move(
    state: &mut State,
    mut ch: RespChannel,
    session_id: SessionId,
    user_id: UserId,
    change: String,
) -> Option<()> {
    let msg_resp = {
        let session = state.sessions.get_mut(&session_id)?;
        if session.move_piece(&user_id, change) {
            MsgResp::Moved
        } else {
            MsgResp::MoveFailure
        }
    };
    ch.send(msg_resp).await;
    Some(())
}

// Handle Reconnect messages

async fn handle_reconnect(
    state: &mut State,
    mut ch: RespChannel,
    session_id: SessionId,
    user_id: UserId,
) -> Option<()> {
    let msg_resp = MsgResp::Reconnected(
        session_id.clone(),
        state
            .sessions
            .get(&session_id)?
            .get_user_name(&user_id)?
            .into(),
    );
    ch.send(msg_resp).await;
    Some(())
}

// Handle Start messages

async fn handle_start(
    state: &mut State,
    mut ch: RespChannel,
    session_id: SessionId,
    user_id: UserId,
) -> Option<()> {
    let msg_resp = {
        let session = state.sessions.get_mut(&session_id)?;
        if session.start(&user_id) {
            MsgResp::Started
        } else {
            MsgResp::StartFailure
        }
    };
    ch.send(msg_resp).await;
    Some(())
}

// Helper functions

fn auth(state: &State, user_id: &UserId, auth_token: &AuthToken) -> Option<SessionId> {
    let tok = state.auth_tokens.get(user_id)?;
    if tok == auth_token {
        Some(state.session_ids.get(user_id)?.clone())
    } else {
        None
    }
}
