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
    Connect {
        session_id: String,
        user_name: String,
    },
    #[serde(rename_all = "camelCase")]
    Create { user_name: String },
    #[serde(rename_all = "camelCase")]
    DeployPiece {
        auth_token: String,
        piece: String,
        pos: String,
    },
    #[serde(rename_all = "camelCase")]
    MovePiece { auth_token: String, change: String },
    #[serde(rename_all = "camelCase")]
    Reconnect { auth_token: String },
    #[serde(rename_all = "camelCase")]
    SetParticipants {
        auth_token: String,
        participants: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    Start { auth_token: String },
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
