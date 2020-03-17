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
    owner: UserId,
    user_names: HashMap<UserId, String>,
    participants: Vec<UserId>,
    started: bool,
    logic: ChessLogic,
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

    pub fn get_user_name(&self, user_id: &UserId) -> Option<&String> {
        self.user_names.get(user_id)
    }

    pub fn set_user_name(&mut self, user_id: UserId, user_name: String) {
        self.user_names.insert(user_id.clone(), user_name.clone());
    }

    pub fn set_participants(&mut self, owner: &UserId, participants: Vec<UserId>) -> bool {
        if *owner != self.owner
            || self.started
            || participants
                .iter()
                .any(|p| self.user_names.get(p).is_none())
        {
            return false;
        }
        self.participants = participants;
        true
    }

    pub fn start(&mut self, owner: &UserId) -> bool {
        if *owner == self.owner && self.participants.len() >= 4 {
            self.started = true;
        }
        self.started
    }

    pub fn move_piece(&mut self, old_pos: String, new_pos: String) -> bool {
        false
    }
}
