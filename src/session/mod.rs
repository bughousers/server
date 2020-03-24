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

mod utils;

use crate::common::*;
use crate::dispatcher::Error;
use bughouse_rs::infoCourier::infoCourier::gen_yfen;
use bughouse_rs::logic::{ChessLogic, Winner};
use futures::lock::{Mutex, MutexGuard};
use serde::Serialize;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::delay_for;

const BROADCAST_CHANNEL_CAPACITY: usize = 5;
const BROADCAST_INTERVAL: Duration = Duration::from_secs(20);
const BROADCAST_MAX_FAILURE: usize = 20;
const GAME_DURATION: Duration = Duration::from_secs(300);
const MAX_NUM_OF_PARTICIPANTS: usize = 5;
const MAX_NUM_OF_USERS: usize = std::u8::MAX as usize + 1;
const TICK: Duration = Duration::from_secs(2);
const ZERO_SECS: Duration = Duration::from_secs(0);

#[derive(Clone)]
pub struct Session {
    inner: Arc<Mutex<SessionInner>>,
}

impl Session {
    pub fn new(owner_name: &str) -> Option<(Session, AuthToken)> {
        let (inner, auth_token) = SessionInner::new(owner_name)?;
        Some((
            Self {
                inner: Arc::new(Mutex::new(inner)),
            },
            auth_token,
        ))
    }

    pub async fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(MutexGuard<SessionInner>) -> R,
    {
        let inner = self.inner.lock().await;
        f(inner)
    }

    pub fn tick(&self) {
        let session = self.clone();
        tokio::spawn(async move {
            let mut i = ZERO_SECS;
            loop {
                delay_for(TICK).await;
                i += TICK;
                session.with(|mut s| s.check_end_conditions()).await;
                if i >= BROADCAST_INTERVAL {
                    let is_alive = session.with(|mut s| s.notify_all()).await;
                    if !is_alive {
                        break;
                    }
                    i = ZERO_SECS;
                }
            }
        });
    }
}

#[derive(Serialize)]
pub struct SessionInner {
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

impl SessionInner {
    pub fn new(owner_name: &str) -> Option<(SessionInner, AuthToken)> {
        if !utils::is_valid_user_name(owner_name) {
            return None;
        }
        let (tx, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        let mut session = Self {
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
            broadcast_tx: tx,
            failed_broadcasts: 0,
        };
        let (_, auth_token) = session.add_user(owner_name).ok()?;
        Some((session, auth_token))
    }

    pub fn get_user_id(&self, auth_token: &AuthToken) -> Option<UserId> {
        self.user_ids.get(auth_token).cloned()
    }

    pub fn get_user_name(&self, user_id: &UserId) -> Option<&String> {
        self.user_names.get(user_id)
    }

    pub fn add_user(&mut self, name: &str) -> Result<(UserId, AuthToken), Error> {
        if !utils::is_valid_user_name(name) {
            return Err(Error::InvalidUserName);
        }
        let user_id = self.user_ids.len();
        if user_id >= MAX_NUM_OF_USERS {
            return Err(Error::TooManyUsers);
        }
        let user_id = UserId::new(user_id as u8);
        let auth_token = AuthToken::new();
        self.user_ids.insert(auth_token.clone(), user_id);
        self.user_names.insert(user_id, name.to_owned());
        self.notify_all();
        Ok((user_id, auth_token))
    }

    fn is_owner(&self, auth_token: &AuthToken) -> bool {
        let user_id = self.get_user_id(auth_token);
        if let Some(user_id) = user_id {
            user_id == UserId::OWNER
        } else {
            false
        }
    }

    pub fn set_participants(
        &mut self,
        auth_token: &AuthToken,
        participants: Vec<UserId>,
    ) -> Result<(), Error> {
        if self.game_id != 0 || self.active_participants.is_some() {
            return Err(Error::GameHasAlreadyStarted);
        } else if !self.is_owner(auth_token) {
            return Err(Error::MustBeSessionOwner);
        } else if participants
            .iter()
            .any(|p| self.user_names.get(p).is_none())
        {
            return Err(Error::InvalidParticipantList);
        }
        self.participants = participants;
        self.notify_all();
        Ok(())
    }

    pub fn start(&mut self, auth_token: &AuthToken) -> Result<(), Error> {
        if self.active_participants.is_some() {
            return Err(Error::GameHasAlreadyStarted);
        } else if !self.is_owner(auth_token) {
            return Err(Error::MustBeSessionOwner);
        } else if self.participants.len() < 4 {
            return Err(Error::NotEnoughParticipants);
        } else if self.participants.len() > MAX_NUM_OF_PARTICIPANTS {
            return Err(Error::TooManyParticipants);
        }
        if self.game_id == 0 {
            let pairings = utils::create_pairings(self.participants.len() as u8);
            self.queue = pairings
                .iter()
                .map(|&((a, b), (c, d))| {
                    (
                        (
                            self.participants[(a - 1) as usize],
                            self.participants[(b - 1) as usize],
                        ),
                        (
                            self.participants[(c - 1) as usize],
                            self.participants[(d - 1) as usize],
                        ),
                    )
                })
                .collect();
        }
        let active_participants = self.queue.pop_front().ok_or(Error::SessionHasEnded)?;
        self.active_participants = Some(active_participants);
        self.notify_all();
        self.reset();
        Ok(())
    }

    fn get_board_and_color(&self, user_id: &UserId) -> Option<(bool, bool)> {
        let ((a, b), (c, _)) = self.active_participants?;
        if a == *user_id {
            Some((true, true))
        } else if b == *user_id {
            Some((false, false))
        } else if c == *user_id {
            Some((true, false))
        } else {
            Some((false, true))
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
        self.logic.refresh();
        let now = Instant::now();
        self.clock = Some(((now, now), (now, now)));
    }

    pub fn deploy_piece(
        &mut self,
        auth_token: &AuthToken,
        piece: String,
        pos: String,
    ) -> Result<(), Error> {
        if self.active_participants.is_none() {
            return Err(Error::GameHasNotStartedYet);
        }
        let user_id = self
            .user_ids
            .get(auth_token)
            .ok_or(Error::InvalidAuthToken)?;
        let (b1, w) = self
            .get_board_and_color(user_id)
            .ok_or(Error::NotAnActiveParticipant)?;
        let piece = utils::parse_piece(&piece).ok_or(Error::CannotParse)?;
        let (col, row) = utils::parse_pos(&pos).ok_or(Error::CannotParse)?;
        if !self.logic.deploy_piece(b1, w, piece, row, col) {
            return Err(Error::IllegalMove);
        } else {
            self.notify_all();
            self.update_clocks(b1);
            self.check_end_conditions();
        }
        Ok(())
    }

    pub fn move_piece(&mut self, auth_token: &AuthToken, change: String) -> Result<(), Error> {
        if self.active_participants.is_none() {
            return Err(Error::GameHasNotStartedYet);
        }
        let user_id = self
            .user_ids
            .get(auth_token)
            .ok_or(Error::InvalidAuthToken)?;
        let (b1, w) = self
            .get_board_and_color(user_id)
            .ok_or(Error::NotAnActiveParticipant)?;
        let is_whites_turn = if b1 {
            self.logic.white_active_1
        } else {
            self.logic.white_active_2
        };
        if is_whites_turn != w {
            return Err(Error::IllegalMove);
        }
        let [i, j, i_new, j_new] = utils::parse_change(&change);
        if !self.logic.movemaker(b1, i, j, i_new, j_new) {
            return Err(Error::IllegalMove);
        } else {
            self.notify_all();
            self.update_clocks(b1);
            self.check_end_conditions();
        }
        Ok(())
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

    pub fn check_end_conditions(&mut self) {
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

    pub fn subscribe(&mut self) -> broadcast::Receiver<String> {
        self.broadcast_tx.subscribe()
    }

    pub fn is_alive(&self) -> bool {
        self.failed_broadcasts <= BROADCAST_MAX_FAILURE
    }

    pub fn notify_all(&mut self) -> bool {
        let board = gen_yfen(&self.logic);
        let ev = Event {
            session: self,
            board,
        };
        let ev = format!("data: {}\n\n", serde_json::to_string(&ev).unwrap());
        match self.broadcast_tx.send(ev) {
            Ok(_) => self.failed_broadcasts = 0,
            _ => self.failed_broadcasts += 1,
        }
        self.is_alive()
    }
}

#[derive(Serialize)]
struct Event<'a> {
    #[serde(flatten)]
    session: &'a SessionInner,
    board: (String, String),
}
