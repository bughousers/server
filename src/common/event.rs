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

use super::data::UserId;
use crate::session::{Game, Session};
use bughouse_rs::infoCourier::infoCourier::gen_yfen;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event<'a> {
    pub caused_by: UserId,
    pub ev: EventType,
    pub session: &'a Session,
}

impl<'a> Event<'a> {
    pub fn to_message(&self) -> String {
        let msg = serde_json::ser::to_string(self).unwrap();
        format!("data: {}\n\n", msg)
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EventType {
    GameEnded(Option<(UserId, UserId)>),
    GameStarted,
    Joined,
    ParticipantsChanged,
    Periodic,
    PieceDeployed,
    PieceMoved,
    PiecePromoted,
}

impl Serialize for Game {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut game = serializer.serialize_struct("Game", 4)?;
        game.serialize_field("activeParticipants", &self.active_participants)?;
        game.serialize_field("remainingTime", &self.remaining_time)?;
        game.serialize_field("board", &gen_yfen(&self.logic))?;
        game.serialize_field("pool", &self.logic.get_pools())?;
        game.end()
    }
}
