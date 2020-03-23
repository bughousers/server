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

use super::data::{AuthToken, SessionId, UserId};
use serde::{Deserialize, Serialize};

/// `Created` is sent when a session is successfully created as per user
/// request.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Created {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub auth_token: AuthToken,
}

/// `Joined` is sent when a user succesfully joins a session.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Joined {
    pub user_id: UserId,
    pub user_name: String,
    pub auth_token: AuthToken,
}

/// `Err` is sent when the server fails to fulfill a user request.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub error: String,
}
