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

use crate::{
    common::event::{Event, EventType},
    common::*,
    config::Config,
};
use bughouse_rs::logic::{ChessLogic, Winner};
use futures::{channel::mpsc, select, FutureExt, StreamExt};
pub use handler::Msg;
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::broadcast, time::interval};

const BROADCAST_CHANNEL_CAPACITY: usize = 5;
const BROADCAST_MAX_FAILURE: usize = 20;
const GAME_DURATION: Duration = Duration::from_secs(300);
const PROMOTE_ADDED_TIME: Duration = Duration::from_secs(3);
const ZERO_SECS: Duration = Duration::from_secs(0);

type Result<T> = std::result::Result<T, Error>;

enum Error {
    Error,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    id: SessionId,
    #[serde(skip_serializing)]
    rx: mpsc::Receiver<Msg>,
    #[serde(skip_serializing)]
    user_ids: HashMap<AuthToken, UserId>,
    user_names: HashMap<UserId, String>,
    participants: Vec<UserId>,
    score: HashMap<UserId, usize>,
    #[serde(skip_serializing)]
    queue: VecDeque<((UserId, UserId), (UserId, UserId))>,
    game_id: usize,
    game: Option<Game>,
    #[serde(skip_serializing)]
    broadcast_tx: broadcast::Sender<String>,
    #[serde(skip_serializing)]
    failed_broadcasts: usize,
    #[serde(skip_serializing)]
    config: Arc<Config>,
}

impl Session {
    pub fn new(
        config: Arc<Config>,
        session_id: SessionId,
        owner_name: &str,
    ) -> Option<(Session, mpsc::Sender<Msg>)> {
        if !utils::is_valid_user_name(owner_name) {
            return None;
        }
        let (tx, rx) = mpsc::channel(config.session_capacity());
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        let session = Self {
            id: session_id,
            rx,
            user_ids: HashMap::with_capacity(0),
            user_names: HashMap::with_capacity(0),
            participants: Vec::with_capacity(0),
            score: HashMap::with_capacity(0),
            queue: VecDeque::with_capacity(0),
            game_id: 0,
            game: None,
            broadcast_tx,
            failed_broadcasts: 0,
            config,
        };
        Some((session, tx))
    }

    pub fn spawn(mut self) {
        tokio::spawn(async move {
            let mut timer = interval(self.config.tick());
            let mut broadcast_timer = interval(self.config.broadcast_interval());
            loop {
                select! {
                    msg = self.rx.next() => {
                        match msg {
                            Some(msg) => handler::handle_msg(&mut self, msg).await,
                            _ => break
                        }
                    },
                    _ = timer.tick().fuse() => handler::handle_timer(&mut self),
                    _ = broadcast_timer.tick().fuse() => handler::handle_broadcast_timer(&mut self),
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

    fn user_id(&self, auth_token: &AuthToken) -> Option<UserId> {
        self.user_ids.get(auth_token).cloned()
    }

    fn add_user(&mut self, name: String) -> Result<(UserId, AuthToken)> {
        if !utils::is_valid_user_name(&name) || self.user_ids.len() >= self.config.max_user() {
            return Err(Error::Error);
        }
        let user_id = UserId::new(self.user_ids.len() as u8);
        let auth_token = AuthToken::new();
        self.user_ids.insert(auth_token.clone(), user_id);
        self.user_names.insert(user_id, name);
        Ok((user_id, auth_token))
    }

    fn set_participants(&mut self, participants: Vec<UserId>) -> Result<()> {
        if self.did_tournament_start()
            || participants
                .iter()
                .any(|p| self.user_names.get(p).is_none())
        {
            return Err(Error::Error);
        }
        self.participants = participants;
        Ok(())
    }

    fn fill_queue(&mut self) -> Result<()> {
        if self.queue.len() > 0 {
            return Ok(());
        } else if self.did_tournament_start() {
            return Err(Error::Error);
        }
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
        Ok(())
    }

    fn did_tournament_start(&self) -> bool {
        self.game_id != 0
    }

    fn did_game_start(&self) -> bool {
        self.game.is_some()
    }

    fn start_game(&mut self) -> Result<()> {
        if self.participants.len() < 4
            || self.participants.len() > self.config.max_participant()
            || self.did_game_start()
        {
            return Err(Error::Error);
        }
        self.fill_queue()?;
        let active_participants = self.queue.pop_front().ok_or(Error::Error)?;
        self.game_id += 1;
        self.game = Some(Game::new(active_participants));
        Ok(())
    }

    fn tick(&mut self) {
        self.game.as_mut().map(|g| {
            g.update_remaining_time(true);
            g.refresh_clock(true);
            g.update_remaining_time(false);
            g.refresh_clock(false);
        });
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
                    self.game = None;
                    self.notify_all(
                        u1,
                        EventType::GameEnded {
                            winners: Some((u1, u2)),
                        },
                    );
                }
                Winner::B1 | Winner::W2 => {
                    self.score
                        .insert(u3, *self.score.get(&u3).unwrap_or(&0) + 1);
                    self.score
                        .insert(u4, *self.score.get(&u4).unwrap_or(&0) + 1);
                    self.game = None;
                    self.notify_all(
                        u3,
                        EventType::GameEnded {
                            winners: Some((u3, u4)),
                        },
                    );
                }
                Winner::P => {
                    self.game = None;
                    self.notify_all(UserId::OWNER, EventType::GameEnded { winners: None });
                }
                _ => (),
            }
        }
    }

    fn notify_all(&mut self, caused_by: UserId, ev: EventType) {
        let ev = Event {
            caused_by,
            ev,
            session: &self,
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

    fn refresh_clock(&mut self, board: bool) {
        let (c1, c2) = &mut self.clock;
        let (c, _) = if board { c1 } else { c2 };
        *c = Instant::now();
    }

    fn extend_remaining_time(&mut self, board: bool, duration: Duration) {
        let ((r1, r2), (r3, r4)) = &mut self.remaining_time;
        let (rw, rb) = if board { (r1, r3) } else { (r4, r2) };
        let r = if self.logic.get_white_active(board) {
            rw
        } else {
            rb
        };
        *r += duration;
    }

    fn update_remaining_time(&mut self, board: bool) {
        let (c1, c2) = &mut self.clock;
        let (c, p) = if board { c1 } else { c2 };
        if *p {
            return;
        }
        let ((r1, r2), (r3, r4)) = &mut self.remaining_time;
        let (rw, rb) = if board { (r1, r3) } else { (r4, r2) };
        let r = if self.logic.get_white_active(board) {
            rw
        } else {
            rb
        };
        *r = r.checked_sub(c.elapsed()).unwrap_or(ZERO_SECS);
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

    fn resign(&mut self, user_id: &UserId) -> Result<()> {
        let (b, w) = self.board_and_color(user_id).ok_or(Error::Error)?;
        self.logic.resign(b, w);
        Ok(())
    }

    fn deploy_piece(&mut self, user_id: &UserId, piece: &str, pos: &str) -> Result<()> {
        let (b1, w) = self.board_and_color(user_id).ok_or(Error::Error)?;
        let piece = utils::parse_piece(piece).ok_or(Error::Error)?;
        let (col, row) = utils::parse_pos(&pos).ok_or(Error::Error)?;
        self.update_remaining_time(b1);
        self.refresh_clock(b1);
        self.logic
            .deploy_piece(b1, w, piece, row, col)
            .or(Err(Error::Error))?;
        self.refresh_clock(b1);
        Ok(())
    }

    fn move_piece(&mut self, user_id: &UserId, change: &str) -> Result<()> {
        let (b1, w) = self.board_and_color(user_id).ok_or(Error::Error)?;
        if self.logic.get_white_active(b1) != w {
            return Err(Error::Error);
        }
        let [i, j, i_new, j_new] = utils::parse_change(&change.to_owned()).ok_or(Error::Error)?;
        self.update_remaining_time(b1);
        self.refresh_clock(b1);
        self.logic
            .movemaker(b1, i, j, i_new, j_new)
            .or(Err(Error::Error))?;
        self.refresh_clock(b1);
        Ok(())
    }

    fn promote_piece(&mut self, user_id: &UserId, change: &str, upgrade_to: &str) -> Result<()> {
        let (b1, w) = self.board_and_color(user_id).ok_or(Error::Error)?;
        if self.logic.get_white_active(b1) != w {
            return Err(Error::Error);
        }
        let [i, j, i_new, j_new] = utils::parse_change(&change.to_owned()).ok_or(Error::Error)?;
        let upgrade_to = utils::parse_piece(&upgrade_to).ok_or(Error::Error)?;
        self.logic.set_promotion(b1, upgrade_to);
        self.extend_remaining_time(b1, PROMOTE_ADDED_TIME);
        self.update_remaining_time(b1);
        self.refresh_clock(b1);
        self.logic
            .movemaker(b1, i, j, i_new, j_new)
            .or(Err(Error::Error))?;
        self.refresh_clock(b1);
        Ok(())
    }
}
