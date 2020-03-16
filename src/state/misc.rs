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

use std::iter::repeat;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

fn rand_alphanum_string(len: usize) -> String {
    let mut rng = thread_rng();
    repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect()
}

#[derive(Clone, Eq, Hash)]
pub struct SessionId {
    data: String,
}

impl SessionId {
    pub fn new() -> Self {
        Self {
            data: rand_alphanum_string(4),
        }
    }
}

impl PartialEq for SessionId {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Into<String> for SessionId {
    fn into(self) -> String {
        self.data
    }
}

#[derive(Clone, Eq, Hash)]
pub struct PlayerId {
    data: String,
}

impl PlayerId {
    pub fn new() -> Self {
        Self {
            data: rand_alphanum_string(12),
        }
    }
}

impl PartialEq for PlayerId {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Into<String> for PlayerId {
    fn into(self) -> String {
        self.data
    }
}

#[derive(Clone, Eq, Hash)]
pub struct AuthToken {
    data: String,
}

impl AuthToken {
    pub fn new() -> Self {
        Self {
            data: rand_alphanum_string(32),
        }
    }
}

impl PartialEq for AuthToken {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Into<String> for AuthToken {
    fn into(self) -> String {
        self.data
    }
}
