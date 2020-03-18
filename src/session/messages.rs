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

use crate::common::{AuthToken, User, UserId};

pub type ResponseReceiver<T> = oneshot::Receiver<T>;
pub type ResponseSender<T> = oneshot::Sender<T>;

pub enum Message {
    DeployPiece(AuthToken, String, String, ResponseSender<()>),
    GetUser(UserId, ResponseSender<User>),
    GetUserId(AuthToken, ResponseSender<UserId>),
    InsertUser(UserId, User),
    InsertUserId(AuthToken, UserId),
    IsAlive(ResponseSender<bool>),
    MovePiece(AuthToken, String, ResponseSender<()>),
    NotifyAll,
    SetParticipants(AuthToken, Vec<UserId>, ResponseSender<()>),
    Start(AuthToken, ResponseSender<()>),
    Subscribe(ResponseSender<broadcast::Receiver<String>>),
}

impl Message {
    pub fn deploy_piece(
        auth_token: AuthToken,
        piece: String,
        pos: String,
    ) -> (Self, ResponseReceiver<()>) {
        let (tx, rx) = channel();
        (Message::DeployPiece(auth_token, piece, pos, tx), rx)
    }

    pub fn get_user(user_id: UserId) -> (Self, ResponseReceiver<User>) {
        let (tx, rx) = channel();
        (Message::GetUser(user_id, tx), rx)
    }

    pub fn get_user_id(auth_token: AuthToken) -> (Self, ResponseReceiver<UserId>) {
        let (tx, rx) = channel();
        (Message::GetUserId(auth_token, tx), rx)
    }

    pub fn insert_user(user_id: UserId, user: User) -> Self {
        Message::InsertUser(user_id, user)
    }

    pub fn insert_user_id(auth_token: AuthToken, user_id: UserId) -> Self {
        Message::InsertUserId(auth_token, user_id)
    }

    pub fn is_alive() -> (Self, ResponseReceiver<bool>) {
        let (tx, rx) = channel();
        (Message::IsAlive(tx), rx)
    }

    pub fn move_piece(auth_token: AuthToken, change: String) -> (Self, ResponseReceiver<()>) {
        let (tx, rx) = channel();
        (Message::MovePiece(auth_token, change, tx), rx)
    }

    pub fn notify_all() -> Self {
        Message::NotifyAll
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

    pub fn subscribe() -> (Self, ResponseReceiver<broadcast::Receiver<String>>) {
        let (tx, rx) = channel();
        (Message::Subscribe(tx), rx)
    }
}
