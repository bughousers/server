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
    #[serde(flatten)]
    pub ev: EventType<'a>,
}

impl<'a> Event<'a> {
    pub fn to_message(&self) -> String {
        let msg = serde_json::ser::to_string(self).unwrap();
        format!("data: {}\n\n", msg)
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum EventType<'a> {
    Joined(Joined<'a>),
    GameStarted(GameStarted<'a>),
    Board(Board<'a>),
    ParticipantsChanged(ParticipantsChanged<'a>),
    FullSync(&'a Session),
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Joined<'a> {
    pub user_id: UserId,
    pub user_name: &'a str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStarted<'a> {
    pub game_id: usize,
    pub active_participants: &'a ((UserId, UserId), (UserId, UserId)),
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Board<'a> {
    pub board: bool,
    pub ev: BoardType<'a>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum BoardType<'a> {
    Deployed(Deployed<'a>),
    Moved(Moved<'a>),
    Promoted(Promoted<'a>),
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deployed<'a> {
    pub piece: &'a str,
    pub pos: &'a str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Moved<'a> {
    pub change: &'a str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Promoted<'a> {
    pub change: &'a str,
    pub upgrade_to: &'a str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantsChanged<'a> {
    pub participants: &'a [UserId],
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
