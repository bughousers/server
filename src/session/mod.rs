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

mod handler;
mod utils;

use crate::common::event::{Event, EventType};
use crate::common::*;
use bughouse_rs::logic::{ChessLogic, Winner};
use futures::channel::mpsc;
use futures::{select, FutureExt, StreamExt};
pub use handler::Msg;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::interval;

const BROADCAST_CHANNEL_CAPACITY: usize = 5;
const BROADCAST_INTERVAL: Duration = Duration::from_secs(20);
const BROADCAST_MAX_FAILURE: usize = 20;
const CHANNEL_CAPACITY: usize = 0;
const GAME_DURATION: Duration = Duration::from_secs(300);
const MAX_NUM_OF_PARTICIPANTS: usize = 5;
const MAX_NUM_OF_USERS: usize = std::u8::MAX as usize + 1;
const TICK: Duration = Duration::from_secs(2);
const ZERO_SECS: Duration = Duration::from_secs(0);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    id: SessionId,
    #[serde(skip_serializing)]
    rx: mpsc::Receiver<Msg>,
    #[serde(skip_serializing)]
    user_ids: HashMap<AuthToken, UserId>,
    user_names: HashMap<UserId, String>,
    score: HashMap<UserId, usize>,
    participants: Vec<UserId>,
    #[serde(skip_serializing)]
    queue: VecDeque<((UserId, UserId), (UserId, UserId))>,
    game_id: usize,
    #[serde(skip_serializing)]
    game: Option<Game>,
    #[serde(skip_serializing)]
    broadcast_tx: broadcast::Sender<String>,
    #[serde(skip_serializing)]
    failed_broadcasts: usize,
}

impl Session {
    pub fn new(session_id: SessionId, owner_name: &str) -> Option<(Session, mpsc::Sender<Msg>)> {
        if !utils::is_valid_user_name(owner_name) {
            return None;
        }
        let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        let session = Self {
            id: session_id,
            rx,
            user_ids: HashMap::with_capacity(0),
            user_names: HashMap::with_capacity(0),
            score: HashMap::with_capacity(0),
            participants: Vec::with_capacity(0),
            queue: VecDeque::with_capacity(0),
            game_id: 0,
            game: None,
            broadcast_tx,
            failed_broadcasts: 0,
        };
        Some((session, tx))
    }

    pub fn spawn(mut self) {
        tokio::spawn(async move {
            let mut timer = interval(TICK);
            let mut broadcast_timer = interval(BROADCAST_INTERVAL);
            loop {
                select! {
                    msg = self.rx.next() => {
                        match msg {
                            Some(msg) => handler::handle(&mut self, msg).await,
                            _ => break
                        }
                    },
                    _ = timer.tick().fuse() => self.tick(),
                    _ = broadcast_timer.tick().fuse() => self.notify_all()
                }
            }
        });
    }

    fn is_owner(&self, auth_token: &AuthToken) -> bool {
        let user_id = self.user_ids.get(auth_token);
        if let Some(user_id) = user_id {
            *user_id == UserId::OWNER
        } else {
            false
        }
    }

    fn reset_game(&mut self, active_participants: ((UserId, UserId), (UserId, UserId))) {
        self.game = Some(Game::new(active_participants));
    }

    fn tick(&mut self) {
        self.game.as_mut().map(|g| g.update_clock());
        self.check_end_conditions();
    }

    fn check_end_conditions(&mut self) {
        if let Some(g) = self.game.as_ref() {
            let ((u1, u2), (u3, u4)) = g.active_participants;
            match g.winner() {
                Winner::W1 | Winner::B2 => {
                    self.score
                        .insert(u1, *self.score.get(&u1).unwrap_or(&0) + 1);
                    self.score
                        .insert(u2, *self.score.get(&u2).unwrap_or(&0) + 1);
                    self.game_id += 1;
                    self.game = None;
                    self.notify_all();
                }
                Winner::B1 | Winner::W2 => {
                    self.score
                        .insert(u3, *self.score.get(&u3).unwrap_or(&0) + 1);
                    self.score
                        .insert(u4, *self.score.get(&u4).unwrap_or(&0) + 1);
                    self.game_id += 1;
                    self.game = None;
                    self.notify_all();
                }
                Winner::P => {
                    self.game_id += 1;
                    self.game = None;
                    self.notify_all();
                }
                _ => (),
            }
        }
    }

    fn notify_all(&mut self) {
        let ev = Event {
            caused_by: UserId::OWNER,
            ev: EventType::FullSync(self),
        };
        match self.broadcast_tx.send(ev.to_message()) {
            Ok(_) => self.failed_broadcasts = 0,
            _ => self.failed_broadcasts += 1,
        }
        if self.failed_broadcasts > BROADCAST_MAX_FAILURE {
            self.rx.close();
        }
    }
}

/// `Game` holds game related data.
pub struct Game {
    /// Active participants.
    ///
    /// Each `UserId` pair represents a team. Each user in a pair plays against
    /// a user in the same position in the other pair. The player colors are as
    /// follows: ((white, black), (black, white)).
    pub active_participants: ((UserId, UserId), (UserId, UserId)),
    /// For each board, we have a clock, which is used for recalculating the
    /// remaining time of the currently active player. If the `bool` value is
    /// `true`, the clock is paused.
    pub clock: ((Instant, bool), (Instant, bool)),
    /// Remaining time for each user. Follows the same order as
    /// `active_participants`.
    pub remaining_time: ((Duration, Duration), (Duration, Duration)),
    pub logic: ChessLogic,
}

impl Game {
    fn new(active_participants: ((UserId, UserId), (UserId, UserId))) -> Self {
        let now = Instant::now();
        Self {
            active_participants,
            clock: ((now, false), (now, false)),
            remaining_time: (
                (GAME_DURATION, GAME_DURATION),
                (GAME_DURATION, GAME_DURATION),
            ),
            logic: ChessLogic::new(),
        }
    }

    fn board_and_color(&self, user_id: &UserId) -> Option<(bool, bool)> {
        let ((a, b), (c, d)) = self.active_participants;
        if a == *user_id {
            Some((true, true))
        } else if b == *user_id {
            Some((false, false))
        } else if c == *user_id {
            Some((true, false))
        } else if d == *user_id {
            Some((false, true))
        } else {
            None
        }
    }

    fn update_clock(&mut self) {
        let ((c1, p1), (c2, p2)) = &mut self.clock;
        let ((r1, r2), (r3, r4)) = &mut self.remaining_time;
        let r1 = if self.logic.get_white_active(true) {
            r1
        } else {
            r3
        };
        let r2 = if self.logic.get_white_active(false) {
            r4
        } else {
            r2
        };
        if !*p1 {
            *r1 = r1.checked_sub(c1.elapsed()).unwrap_or(ZERO_SECS);
        }
        if !*p2 {
            *r2 = r2.checked_sub(c1.elapsed()).unwrap_or(ZERO_SECS);
        }
        let now = Instant::now();
        *c1 = now;
        *c2 = now;
    }

    fn add_time(&mut self, user_id: &UserId, time: Duration) {
        if let Some(board_and_color) = self.board_and_color(&user_id) {
            let ((r1, r2), (r3, r4)) = &mut self.remaining_time;
            let rem = match board_and_color {
                (true, true) => r1,
                (false, false) => r2,
                (true, false) => r3,
                (false, true) => r4,
            };
            *rem = *rem + time;
        }
    }

    fn winner(&self) -> Winner {
        let ((r1, r2), (r3, r4)) = self.remaining_time;
        if self.logic.get_white_active(true) && r1 == ZERO_SECS {
            Winner::B1
        } else if !self.logic.get_white_active(false) && r2 == ZERO_SECS {
            Winner::W2
        } else if !self.logic.get_white_active(true) && r3 == ZERO_SECS {
            Winner::W1
        } else if self.logic.get_white_active(false) && r4 == ZERO_SECS {
            Winner::B2
        } else {
            self.logic.get_winner(true)
        }
    }
}
