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

use rand::{distributions, thread_rng, Rng};
use std::iter::repeat;

pub fn rand_auth_token() -> String {
    rand_alphanum_string(32)
}

pub fn rand_session_id() -> String {
    rand_alphanum_string(4)
}

fn rand_alphanum_string(len: usize) -> String {
    repeat(())
        .map(|()| thread_rng().sample(distributions::Alphanumeric))
        .take(len)
        .collect()
}
