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

use super::data::{AuthToken, SessionId, User, UserId};
use hyper::Body;
use serde::{Deserialize, Serialize};

/// Enum of request types which the server may receive.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Request {
    /// Authenticated requests will be dispatched to the session associated with
    /// the auth token.
    #[serde(rename_all = "camelCase")]
    Authenticated {
        auth_token: AuthToken,
        data: Authenticated,
    },
    /// The user wants to connect to an already existing session for the first
    /// time.
    #[serde(rename_all = "camelCase")]
    Connect {
        session_id: SessionId,
        user_name: String,
    },
    /// The user wants to create a new session.
    #[serde(rename_all = "camelCase")]
    Create { user_name: String },
}

/// Enum of authenticated request types which the server may receive.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Authenticated {
    /// The user wants to place a piece on the chessboard.
    #[serde(rename_all = "camelCase")]
    DeployPiece { piece: String, pos: String },
    /// The user wants to move a piece.
    #[serde(rename_all = "camelCase")]
    MovePiece { change: String },
    #[serde(rename_all = "camelCase")]
    /// The user wants to re-join a session which they have joined before.
    Reconnect,
    #[serde(rename_all = "camelCase")]
    /// Contains the list of users who will be playing in a match. This request
    /// will be ignored if it isn't sent by the session owner.
    SetParticipants { participants: Vec<UserId> },
    /// Start the game. This request will be ignored if it isn't sent by the
    /// session owner.
    #[serde(rename_all = "camelCase")]
    Start,
}

/// Enum of response types which the server may send back to the client.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Response {
    /// The user has succesfully connected to the requested session.
    #[serde(rename_all = "camelCase")]
    Connected { user: User, auth_token: AuthToken },
    /// The session has been succesfully created.
    #[serde(rename_all = "camelCase")]
    Created { auth_token: AuthToken },
    /// The user has succesfully restored their connection to the session.
    #[serde(rename_all = "camelCase")]
    Reconnected { session_id: SessionId, user: User },
    /// A request which doesn't expect a specific response has been completed
    /// successfully.
    Success,
}

impl Into<Body> for Response {
    fn into(self) -> Body {
        Body::from(serde_json::to_string(&self).expect("Serialization failed"))
    }
}

/// `Event` represents a snapshot of the current state of a session.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub users: Vec<User>,
    pub board: (String, String),
    pub started: bool,
}
