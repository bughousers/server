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

use std::iter::FromIterator;
pub mod pairings;
pub use pairings::create_pairings;

use super::Session;
use crate::common::{User, UserId, UserStatus};
use bughouse_rs::logic::board::Piece;
pub use bughouse_rs::parse::parser::parse as parse_change;
use std::collections::LinkedList;

use std::collections::VecDeque;
use std::option::Option;
use std::ops::Rem;


pub fn validate_user_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}

pub fn parse_piece(s: &String) -> Option<Piece> {
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

pub fn parse_pos(s: &String) -> Option<(usize, usize)> {
    let mut buf = s.bytes();
    let col = buf.next()? as usize;
    let row = buf.next()? as usize;
    if col >= 97 && col <= 104 && row >= 48 && row <= 55 {
        Some((col - 97, row - 48))
    } else {
        None
    }
}

pub fn create_pairings_wr(n:u8) -> VecDeque<((u8,u8),(u8,u8))> {
    if n == 4 {
        let hard_coded = VecDeque::from_iter(vec![   
            ((1,2),(3,4)),
            ((1,2),(4,3)),
            ((2,1),(3,4)),
            ((2,1),(4,3)),
            ((1,3),(2,4)),
            ((1,3),(4,2)),
            ((3,1),(2,4)),
            ((3,1),(4,2)),
            ((2,3),(1,4)),
            ((2,3),(4,1)),
            ((3,2),(1,4)),
            ((3,2),(4,1)),
            ((1,4),(2,3)),
            ((1,4),(3,2)),
            ((4,1),(2,3)),
            ((4,1),(3,2)),
            ((2,4),(1,3)),
            ((2,4),(3,1)),
            ((4,2),(1,3)),
            ((4,2),(3,1)),
            ((3,4),(1,2)),
            ((3,4),(2,1)),
            ((4,3),(1,2)),
            ((4,3),(2,1))
        ]);
       
        hard_coded
    }else{
        let hmph = create_pairings(4);
        hmph
    }
   


}


