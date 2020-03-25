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

use crate::common::*;
use bughouse_rs::infoCourier::infoCourier::gen_yfen;
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

#[derive(Serialize)]
pub struct Session {
    id: SessionId,
    #[serde(skip_serializing)]
    rx: mpsc::Receiver<Msg>,
    #[serde(skip_serializing)]
    user_ids: HashMap<AuthToken, UserId>,
    user_names: HashMap<UserId, String>,
    score: HashMap<UserId, usize>,
    participants: Vec<UserId>,
    active_participants: Option<((UserId, UserId), (UserId, UserId))>,
    #[serde(skip_serializing)]
    queue: VecDeque<((UserId, UserId), (UserId, UserId))>,
    #[serde(skip_serializing)]
    clock: Option<((Instant, Instant), (Instant, Instant))>,
    remaining_time: ((Duration, Duration), (Duration, Duration)),
    #[serde(skip_serializing)]
    logic: ChessLogic,
    game_id: usize,
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
            active_participants: None,
            queue: VecDeque::with_capacity(0),
            clock: None,
            remaining_time: (
                (GAME_DURATION, GAME_DURATION),
                (GAME_DURATION, GAME_DURATION),
            ),
            logic: ChessLogic::new(),
            game_id: 0,
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
                    _ = timer.tick().fuse() => self.check_end_conditions(),
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

    fn get_board_and_color(&self, user_id: &UserId) -> Option<(bool, bool)> {
        let ((a, b), (c, d)) = self.active_participants?;
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

    // Run after each successful move.
    fn update_clocks(&mut self, b1: bool) {
        let now = Instant::now();
        if let Some(((c1, c2), (c3, c4))) = &mut self.clock {
            let ((r1, r2), (r3, r4)) = &mut self.remaining_time;
            if b1 {
                if self.logic.white_active_1 {
                    *c1 = now;
                    *r3 -= now - *c3;
                } else {
                    *c3 = now;
                    *r1 -= now - *c1;
                }
            } else {
                if self.logic.white_active_2 {
                    *c4 = now;
                    *r2 -= now - *c2;
                } else {
                    *c2 = now;
                    *r4 -= now - *c4;
                }
            }
        }
    }

    fn reset(&mut self) {
        let now = Instant::now();
        self.clock = Some(((now, now), (now, now)));
        self.remaining_time = (
            (GAME_DURATION, GAME_DURATION),
            (GAME_DURATION, GAME_DURATION),
        );
        self.logic.refresh();
    }

    fn get_winner(&self) -> Option<Winner> {
        if self.active_participants.is_none() {
            return None;
        }
        if let Some(((c1, c2), (c3, c4))) = self.clock {
            let ((r1, r2), (r3, r4)) = self.remaining_time;
            let now = Instant::now();
            if self.logic.white_active_1 && now - c1 > r1 {
                return Some(Winner::B1);
            } else if !self.logic.white_active_1 && now - c3 > r3 {
                return Some(Winner::W1);
            } else if self.logic.white_active_2 && now - c4 > r4 {
                return Some(Winner::B2);
            } else if !self.logic.white_active_2 && now - c2 > r2 {
                return Some(Winner::W2);
            } else {
                Some(self.logic.get_winner(true))
            }
        } else {
            None
        }
    }

    fn check_end_conditions(&mut self) {
        if let Some(((u1, u2), (u3, u4))) = self.active_participants {
            let winner = self.get_winner();
            match winner {
                Some(Winner::W1) | Some(Winner::B2) => {
                    self.score
                        .insert(u1, *self.score.get(&u1).unwrap_or(&0) + 1);
                    self.score
                        .insert(u2, *self.score.get(&u2).unwrap_or(&0) + 1);
                    self.active_participants = None;
                    self.notify_all();
                }
                Some(Winner::B1) | Some(Winner::W2) => {
                    self.score
                        .insert(u3, *self.score.get(&u3).unwrap_or(&0) + 1);
                    self.score
                        .insert(u4, *self.score.get(&u4).unwrap_or(&0) + 1);
                    self.active_participants = None;
                    self.notify_all();
                }
                Some(Winner::P) => {
                    self.active_participants = None;
                    self.notify_all();
                }
                _ => (),
            }
        }
    }

    fn notify_all(&mut self) {
        let ev = Event {
            session: self,
            board: gen_yfen(&self.logic),
        };
        let ev = format!("data: {}\n\n", serde_json::to_string(&ev).unwrap());
        match self.broadcast_tx.send(ev) {
            Ok(_) => self.failed_broadcasts = 0,
            _ => self.failed_broadcasts += 1,
        }
        if self.failed_broadcasts > BROADCAST_MAX_FAILURE {
            self.rx.close();
        }
    }
}

#[derive(Serialize)]
struct Event<'a> {
    #[serde(flatten)]
    session: &'a Session,
    board: (String, String),
}
