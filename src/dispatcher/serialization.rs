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

use serde::{Deserialize, Serialize};

use super::{json_builder, DispatchResult};

// Request types

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Req {
    #[serde(rename_all = "camelCase")]
    Authenticated {
        auth_token: String,
        data: Authenticated,
    },
    #[serde(rename_all = "camelCase")]
    Connect {
        session_id: String,
        user_name: String,
    },
    #[serde(rename_all = "camelCase")]
    Create { user_name: String },
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Authenticated {
    #[serde(rename_all = "camelCase")]
    Config { data: Config },
    #[serde(rename_all = "camelCase")]
    Move { data: Move },
    #[serde(rename_all = "camelCase")]
    Reconnect,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Config {
    #[serde(rename_all = "camelCase")]
    Participants { participants: Vec<String> },
    #[serde(rename_all = "camelCase")]
    Start,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Move {
    #[serde(rename_all = "camelCase")]
    Deploy { piece: String, pos: String },
    #[serde(rename_all = "camelCase")]
    Move { change: String },
}

// Response types

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Resp {
    #[serde(rename_all = "camelCase")]
    Connected { user_id: String, auth_token: String },
    #[serde(rename_all = "camelCase")]
    Created { auth_token: String },
    #[serde(rename_all = "camelCase")]
    Reconnected {
        session_id: String,
        user_id: String,
        user_name: String,
    },
}

impl Into<DispatchResult> for Resp {
    fn into(self) -> DispatchResult {
        Ok(json_builder().body(hyper::Body::from(serde_json::to_string(&self).unwrap()))?)
    }
}
