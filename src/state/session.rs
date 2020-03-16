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

use bughouse_rs::logic::ChessLogic;

use crate::state::UserId;

pub struct Session {
    pub owner: UserId,
    pub user_names: HashMap<UserId, String>,
    pub participants: Vec<UserId>,
    pub started: bool,
    pub logic: ChessLogic,
}

impl Session {
    pub fn new(owner: UserId) -> Self {
        Self {
            owner,
            user_names: HashMap::new(),
            participants: Vec::with_capacity(4),
            started: false,
            logic: ChessLogic::new(),
        }
    }
}
