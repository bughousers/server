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
