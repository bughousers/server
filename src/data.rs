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

use rand::{thread_rng, Rng};
use serde::Serialize;
use std::{
    fmt,
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(into = "String")]
pub struct Token(u128);

impl Token {
    pub fn new() -> Self {
        Self(thread_rng().gen())
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:032x}", self.0))
    }
}

impl FromStr for Token {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(u128::from_str_radix(s, 16)?))
    }
}

impl Into<String> for Token {
    fn into(self) -> String {
        self.to_string()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct User {
    name: String,
    score: usize,
}

impl User {
    pub fn new(name: String) -> Option<User> {
        if name
            .chars()
            .any(|c| !c.is_alphabetic() && !c.is_whitespace())
        {
            None
        } else {
            Some(User { name, score: 0 })
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn score(&self) -> &usize {
        &self.score
    }

    pub fn score_mut(&mut self) -> &mut usize {
        &mut self.score
    }
}
