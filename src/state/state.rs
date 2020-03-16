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

use crate::state::misc::{AuthToken, PlayerId, SessionId};
use crate::state::session::Session;

pub struct State {
    sessions: HashMap<SessionId, Session>,
    session_ids: HashMap<PlayerId, SessionId>,
    auth_tokens: HashMap<PlayerId, AuthToken>,
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
    Create,
}

pub enum MsgResp {
    Created(SessionId, PlayerId, AuthToken),
}

async fn handle(state: &mut State, msg: Msg) {
    match msg.data {
        MsgData::Create => handle_create(state, msg.resp_channel).await,
    }
}

async fn handle_create(state: &mut State, mut ch: RespChannel) {
    let session_id = SessionId::new();
    let player_id = PlayerId::new();
    let auth_token = AuthToken::new();
    let session = Session::new(player_id.clone());
    state.sessions.insert(session_id.clone(), session);
    state
        .session_ids
        .insert(player_id.clone(), session_id.clone());
    state
        .auth_tokens
        .insert(player_id.clone(), auth_token.clone());
    ch.send(MsgResp::Created(session_id, player_id, auth_token))
        .await;
}
