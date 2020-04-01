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

use super::data::{AuthToken, UserId};
use serde::{Deserialize, Serialize};

/// `Create` is received when the user wants to create a new session.
///
/// API endpoint: `POST /v1/sessions`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Create {
    pub owner_name: String,
}

/// `Delete` is received when the session owner wants to end a session.
///
/// API endpoint: `DELETE /v1/sessions/:sid`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
    pub auth_token: AuthToken,
}

/// `Join` is received when the user wants to join a session.
///
/// API endpoint: `POST /v1/sessions/:sid`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum Join {
    /// The user wants to connect to a session and they already have an
    /// authentication token.
    #[serde(rename_all = "camelCase")]
    Connect { auth_token: AuthToken },
    /// The user wants to join an already existing session for the first time.
    #[serde(rename_all = "camelCase")]
    Join { user_name: String },
}

/// `Start` is received when the session owner wants to start a game.
///
/// API endpoint: `POST /v1/sessions/:sid/games`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Start {
    pub auth_token: AuthToken,
}

/// `Resign` is received when an active participant wants to surrender.
///
/// API endpoint: `POST /v1/sessions/:sid/games/:gid`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resign {
    pub auth_token: AuthToken,
}

/// `Board` is received when the user wants to modify the state of the
/// chessboard.
///
/// API endpoint: `POST /v1/sessions/:sid/games/:gid/board`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Board {
    /// The user wants to place a piece on the chessboard.
    #[serde(rename_all = "camelCase")]
    Deploy {
        auth_token: AuthToken,
        piece: String,
        pos: String,
    },
    /// The user wants to move a piece.
    #[serde(rename_all = "camelCase")]
    Move {
        auth_token: AuthToken,
        change: String,
    },
    /// The user wants to make a move that will result in upgrade of a piece.
    #[serde(rename_all = "camelCase")]
    Promote {
        auth_token: AuthToken,
        change: String,
        upgrade_to: String,
    },
}

/// `Participants` is received when the session owner wants to modify the list
/// of users who will be playing in a match.
///
/// API endpoint: `PUT /v1/sessions/:sid/participants`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Participants {
    pub auth_token: AuthToken,
    pub participants: Vec<UserId>,
}
