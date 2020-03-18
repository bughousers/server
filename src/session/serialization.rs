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

use std::collections::HashMap;

use bughouse_rs::infoCourier::infoCourier::gen_yfen;
use serde::{Deserialize, Serialize};

use crate::common::User;

use super::Session;

#[derive(Clone, Deserialize, Serialize)]
pub struct Event {
    pub users: HashMap<String, User>,
    pub owner_id: String,
    pub started: bool,
    pub boards: [String; 2],
}

impl From<&mut Session> for Event {
    fn from(session: &mut Session) -> Self {
        let (x, y) = gen_yfen(&mut session.logic);
        Event {
            users: session.users.clone(),
            owner_id: session.owner_id.clone(),
            started: session.started,
            boards: [x, y],
        }
    }
}
