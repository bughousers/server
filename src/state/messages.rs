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

use futures::channel::oneshot;
use futures::channel::oneshot::channel;
use tokio::sync::broadcast;

use crate::common::{AuthToken, SessionId, UserId};

pub type ResponseReceiver<T> = oneshot::Receiver<T>;
pub type ResponseSender<T> = oneshot::Sender<T>;

pub enum Message {
    Connect(SessionId, String, ResponseSender<(UserId, AuthToken)>),
    Create(String, ResponseSender<AuthToken>),
    DeployPiece(AuthToken, String, String, ResponseSender<()>),
    GarbageCollect,
    MovePiece(AuthToken, String, ResponseSender<()>),
    Reconnect(AuthToken, ResponseSender<(SessionId, UserId, String)>),
    SetParticipants(AuthToken, Vec<UserId>, ResponseSender<()>),
    Start(AuthToken, ResponseSender<()>),
    Subscribe(SessionId, ResponseSender<broadcast::Receiver<String>>),
}

impl Message {
    pub fn connect(
        session_id: SessionId,
        user_name: String,
    ) -> (Self, ResponseReceiver<(UserId, AuthToken)>) {
        let (tx, rx) = channel();
        (Message::Connect(session_id, user_name, tx), rx)
    }

    pub fn create(user_name: String) -> (Self, ResponseReceiver<AuthToken>) {
        let (tx, rx) = channel();
        (Message::Create(user_name, tx), rx)
    }

    pub fn deploy_piece(
        auth_token: AuthToken,
        piece: String,
        pos: String,
    ) -> (Self, ResponseReceiver<()>) {
        let (tx, rx) = channel();
        (Message::DeployPiece(auth_token, piece, pos, tx), rx)
    }

    pub fn garbage_collect() -> Self {
        Message::GarbageCollect
    }

    pub fn move_piece(auth_token: AuthToken, change: String) -> (Self, ResponseReceiver<()>) {
        let (tx, rx) = channel();
        (Message::MovePiece(auth_token, change, tx), rx)
    }

    pub fn reconnect(
        auth_token: AuthToken,
    ) -> (Self, ResponseReceiver<(SessionId, UserId, String)>) {
        let (tx, rx) = channel();
        (Message::Reconnect(auth_token, tx), rx)
    }

    pub fn set_participants(
        auth_token: AuthToken,
        participants: Vec<UserId>,
    ) -> (Self, ResponseReceiver<()>) {
        let (tx, rx) = channel();
        (Message::SetParticipants(auth_token, participants, tx), rx)
    }

    pub fn start(auth_token: AuthToken) -> (Self, ResponseReceiver<()>) {
        let (tx, rx) = channel();
        (Message::Start(auth_token, tx), rx)
    }

    pub fn subscribe(
        session_id: SessionId,
    ) -> (Self, ResponseReceiver<broadcast::Receiver<String>>) {
        let (tx, rx) = channel();
        (Message::Subscribe(session_id, tx), rx)
    }
}
