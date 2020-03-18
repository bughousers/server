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

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type SessionId = String;
pub type UserId = String;
pub type AuthToken = String;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub struct User {
    pub name: String,
    pub status: UserStatus,
}

impl User {
    pub fn new(name: String) -> Self {
        User {
            name,
            status: UserStatus::Spectator,
        }
    }

    pub fn is_active(&self) -> bool {
        match self.status {
            UserStatus::Active(_, _) => true,
            _ => false,
        }
    }

    pub fn is_inactive(&self) -> bool {
        match self.status {
            UserStatus::Inactive => true,
            _ => false,
        }
    }

    pub fn is_participant(&self) -> bool {
        self.is_active() || self.is_inactive()
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UserStatus {
    Active(bool, bool),
    Inactive,
    Spectator,
}
