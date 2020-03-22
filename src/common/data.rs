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

use super::utils::{rand_auth_token, rand_session_id};
use serde::{Deserialize, Serialize};

/// A unique ID which identifies the session.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionId(String);

impl SessionId {
    pub fn new() -> Self {
        Self(rand_session_id())
    }
}

impl<T: Into<String>> From<T> for SessionId {
    fn from(t: T) -> Self {
        Self(t.into())
    }
}

/// An ID which identifies a user in a session. **Not** globally unique.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserId(u8);

impl UserId {
    pub const OWNER: UserId = UserId(0);

    pub fn new(id: u8) -> Self {
        Self(id)
    }
}

/// `AuthToken` lets us verify a request's authenticity. This token should be
/// kept secret between the user and the server.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthToken(String);

impl AuthToken {
    pub fn new() -> Self {
        Self(rand_auth_token())
    }
}

/// A struct which contains user information. `User` should only hold data that
/// is meant to be public (i.e., not the auth token).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub status: UserStatus,
}

impl User {
    pub fn new(id: UserId, name: String) -> Self {
        Self {
            id,
            name,
            status: UserStatus::Spectator,
        }
    }

    pub fn is_participant(&self) -> bool {
        self.status == UserStatus::Inactive
            || if let UserStatus::Active(_, _) = self.status {
                true
            } else {
                false
            }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UserStatus {
    /// User is an active player in the current match.
    Active(bool, bool),
    /// User is **not** an active player in the current match, but they will
    /// play in an upcoming match.
    Inactive,
    /// User is a spectator. They will never be an active player in a match.
    Spectator,
}
