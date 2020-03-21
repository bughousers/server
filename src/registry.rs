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

use crate::common::*;
use crate::dispatcher::{Message as DispatcherMessage, MessageError as DispatcherMessageError};
use crate::session::{Message as SessionMessage, Session};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot::Sender;

const CHANNEL_CAPACITY: usize = 20;

/// Enum of message types which `Registry` can handle.
#[derive(Debug)]
pub enum Message {
    /// Remove the session entry associated with the given `SessionId`.
    Deregister(SessionId),
    /// Associate a user with a session.
    Register(AuthToken, SessionId),
    /// Directly relay the message to a session.
    Relay(SessionId, SessionMessage),
    /// `Request` represents a user request.
    ///
    /// Authenticated requests are directly passed on to the session, whereas
    /// Connect and Create requests are processed locally as well.
    ///
    /// This message contains a channel sender to enable bidirectional
    /// communication with the dispatcher.
    Request(Request, Sender<DispatcherMessage>),
}

/// `Registry` is meant to facilitate communication between the dispatcher and a
/// session.
pub struct Registry {
    session_ids: HashMap<AuthToken, SessionId>,
    sessions: HashMap<SessionId, mpsc::Sender<SessionMessage>>,
    handle: mpsc::Sender<Message>,
}

impl Registry {
    /// Create an instance of `Registry` and start processing messages in the
    /// background.
    pub fn spawn() -> mpsc::Sender<Message> {
        let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
        let registry = Self {
            session_ids: HashMap::new(),
            sessions: HashMap::new(),
            handle: tx.clone(),
        };
        tokio::spawn(async move {
            let (mut registry, mut rx) = (registry, rx);
            while let Some(msg) = rx.recv().await {
                handle(&mut registry, msg).await;
            }
        });
        tx
    }
}

async fn handle(r: &mut Registry, msg: Message) {
    match msg {
        Message::Deregister(sid) => handle_deregister(r, sid).await,
        Message::Register(tok, sid) => handle_register(r, tok, sid).await,
        Message::Relay(sid, msg) => handle_relay(r, sid, msg).await,
        Message::Request(req, tx) => handle_request(r, req, tx).await,
    }
}

async fn handle_deregister(r: &mut Registry, session_id: SessionId) {
    r.session_ids = r
        .session_ids
        .iter()
        .filter(|(_, sid)| **sid != session_id)
        .map(|(tok, sid)| (tok.clone(), sid.clone()))
        .collect();
    r.sessions.remove(&session_id);
}

async fn handle_register(r: &mut Registry, auth_token: AuthToken, session_id: SessionId) {
    r.session_ids.insert(auth_token, session_id);
}

async fn handle_relay(r: &mut Registry, session_id: SessionId, msg: SessionMessage) {
    if let Some(handle) = r.sessions.get_mut(&session_id) {
        let _ = handle.send(msg).await;
    }
}

async fn handle_request(r: &mut Registry, req: Request, tx: Sender<DispatcherMessage>) {
    match req {
        Request::Authenticated { auth_token, data } => {
            handle_authenticated(r, auth_token, data, tx).await
        }
        Request::Connect {
            session_id,
            user_name,
        } => handle_connect(r, session_id, user_name, tx).await,
        Request::Create { user_name } => handle_create(r, user_name, tx).await,
    }
}

async fn handle_authenticated(
    r: &mut Registry,
    auth_token: AuthToken,
    data: Authenticated,
    tx: Sender<DispatcherMessage>,
) {
    if let Some(session_id) = r.session_ids.get(&auth_token) {
        // We use `unwrap()` here because if a user is associated with a session
        // despite the session not existing, the program is already in an
        // inconsistent state.
        let handle = r.sessions.get_mut(session_id).unwrap();
        let _ = handle
            .send(SessionMessage::Request(
                Request::Authenticated { auth_token, data },
                tx,
            ))
            .await;
    } else {
        let _ = tx.send(DispatcherMessage::Error(
            DispatcherMessageError::AuthTokenInvalid,
        ));
    }
}

async fn handle_connect(
    r: &mut Registry,
    session_id: SessionId,
    user_name: String,
    tx: Sender<DispatcherMessage>,
) {
    if !validate_user_name(&user_name) {
        let _ = tx.send(DispatcherMessage::Error(
            DispatcherMessageError::UserNameInvalid,
        ));
        return;
    }
    if let Some(handle) = r.sessions.get_mut(&session_id) {
        let _ = handle
            .send(SessionMessage::Request(
                Request::Connect {
                    session_id,
                    user_name,
                },
                tx,
            ))
            .await;
    } else {
        let _ = tx.send(DispatcherMessage::Error(
            DispatcherMessageError::SessionIdInvalid,
        ));
    }
}

async fn handle_create(r: &mut Registry, user_name: String, tx: Sender<DispatcherMessage>) {
    if !validate_user_name(&user_name) {
        let _ = tx.send(DispatcherMessage::Error(
            DispatcherMessageError::UserNameInvalid,
        ));
        return;
    }
    let session_id = SessionId::new();
    let mut handle = Session::spawn(session_id.clone(), r.handle.clone());
    if handle
        .send(SessionMessage::Request(Request::Create { user_name }, tx))
        .await
        .is_ok()
    {
        r.sessions.insert(session_id, handle);
    }
}

// Helper functions

pub fn validate_user_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}
