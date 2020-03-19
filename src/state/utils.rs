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

use rand::Rng;

use crate::common::{AuthToken, SessionId, UserId};

pub fn rand_auth_token() -> AuthToken {
    rand_alphanum_string(32)
}

pub fn rand_user_id() -> UserId {
    rand_alphanum_string(16)
}

pub fn rand_session_id() -> SessionId {
    rand_alphanum_string(4)
}

fn rand_alphanum_string(len: usize) -> String {
    std::iter::repeat(())
        .map(|()| rand::thread_rng().sample(rand::distributions::Alphanumeric))
        .take(len)
        .collect()
}
