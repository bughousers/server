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
use crate::session::Session;
use serde::Serialize;

/// `Created` is sent when a session is successfully created as per user
/// request.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Created<'a> {
    pub session_id: &'a SessionId,
    pub auth_token: &'a AuthToken,
}

/// `Joined` is sent when a user succesfully joins a session for the first time.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Joined<'a> {
    pub auth_token: &'a AuthToken,
}

/// `Connected` is sent when a user succesfully connects to a session.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Connected<'a> {
    pub user_id: &'a UserId,
    pub session: &'a Session,
}
