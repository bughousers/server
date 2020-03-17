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
use bughouse_rs::logic::board::Piece;
use bughouse_rs::logic::ChessLogic;
use bughouse_rs::parse::parser::parse as parse_change;
use tokio::sync::broadcast::{channel, Receiver, Sender};

use crate::dispatcher::serialization::Event;
use crate::state::UserId;
use crate::ServerError;

#[derive(Clone, Copy)]
enum Color {
    Black,
    White,
}

impl Into<String> for Color {
    fn into(self) -> String {
        match self {
            Color::Black => "black".into(),
            Color::White => "white".into(),
        }
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Color::Black, Color::White) => false,
            (Color::White, Color::Black) => false,
            _ => true,
        }
    }
}

pub struct Session {
    owner: UserId,
    user_names: HashMap<UserId, String>,
    participants: Vec<UserId>,
    active_participants: HashMap<UserId, (usize, Color)>,
    started: bool,
    logic: ChessLogic,
    tx: Sender<String>,
    rx: Receiver<String>,
}

impl Session {
    pub fn new(owner: UserId) -> Self {
        let (tx, rx) = channel(16);
        Self {
            owner,
            user_names: HashMap::new(),
            participants: Vec::with_capacity(4),
            active_participants: HashMap::with_capacity(4),
            started: false,
            logic: ChessLogic::new(),
            tx,
            rx,
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

    pub fn deploy_piece(&mut self, user_id: &UserId, piece: String, pos: String) -> Option<()> {
        let (b, c) = &self.active_participants.get(user_id)?;
        let (col, row) = parse_pos(&pos)?;
        if self
            .logic
            .deploy_piece(*b == 1, *c == Color::White, parse_piece(&piece)?, row, col)
        {
            Some(())
        } else {
            None
        }
    }

    pub fn move_piece(&mut self, user_id: &UserId, change: String) -> bool {
        if let Some((b, c)) = &self.active_participants.get(user_id) {
            let is_white_active = if *b == 1 {
                self.logic.white_active_1
            } else {
                self.logic.white_active_2
            };
            if (*c == Color::White) != is_white_active {
                return false;
            }
            let [i, j, i_new, j_new] = parse_change(&change);
            self.logic.movemaker(*b == 1, i, j, i_new, j_new)
        } else {
            false
        }
    }

    // TODO: Move this to a more appropriate
    // module.
    pub fn notify_all(&mut self) -> Result<(), ServerError> {
        let (fen1, fen2) = gen_yfen(&mut self.logic);
        let ev = Event {
            owner: self.owner.clone().into(),
            user_names: self
                .user_names
                .iter()
                .map(|(k, v)| (k.clone().into(), v.clone()))
                .collect(),
            participants: self.participants.iter().map(|p| p.clone().into()).collect(),
            active_participants: self
                .active_participants
                .iter()
                .map(|(k, (b, c))| (k.clone().into(), (*b, (*c).into())))
                .collect(),
            started: self.started,
            boards: vec![fen1, fen2],
        };
        let json = serde_json::to_string(&ev)?;
        self.tx.send(format!("data: {}", json));
        Ok(())
    }

    pub fn subscribe(&mut self) -> Receiver<String> {
        self.tx.subscribe()
    }
}

// Helper functions

fn parse_piece(s: &String) -> Option<Piece> {
    match s.as_str() {
        "b" => Some(Piece::b),
        "B" => Some(Piece::B),
        "E" => Some(Piece::E),
        "k" => Some(Piece::k),
        "K" => Some(Piece::K),
        "L" => Some(Piece::L),
        "n" => Some(Piece::n),
        "N" => Some(Piece::N),
        "p" => Some(Piece::p),
        "P" => Some(Piece::P),
        "q" => Some(Piece::q),
        "Q" => Some(Piece::Q),
        "r" => Some(Piece::r),
        "R" => Some(Piece::R),
        "Ub" => Some(Piece::Ub),
        "UB" => Some(Piece::UB),
        "Un" => Some(Piece::Un),
        "UN" => Some(Piece::UN),
        "Uq" => Some(Piece::Uq),
        "UQ" => Some(Piece::UQ),
        "Ur" => Some(Piece::Ur),
        "UR" => Some(Piece::UR),
        _ => None,
    }
}

fn parse_pos(s: &String) -> Option<(usize, usize)> {
    let mut buf = s.bytes();
    let col = buf.next()? as usize;
    let row = buf.next()? as usize;
    if col >= 97 && col <= 104 && row >= 48 && row <= 55 {
        Some((col - 97, row - 48))
    } else {
        None
    }
}
