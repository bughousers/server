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

use super::utils;
use super::Session;
use super::{MAX_NUM_OF_PARTICIPANTS, MAX_NUM_OF_USERS};
use crate::common::req::*;
use crate::common::resp::*;
use crate::common::*;
use futures::channel::oneshot;
use tokio::sync::broadcast;

type Result = std::result::Result<(), ()>;

pub enum Msg {
    C(Create, oneshot::Sender<Created>),
    D(Delete),
    J(Join, oneshot::Sender<Joined>),
    S(Start),
    B(Board),
    P(Participants),
    Subscribe(oneshot::Sender<broadcast::Receiver<String>>),
}

pub async fn handle(s: &mut Session, msg: Msg) {
    let _ = match msg {
        Msg::C(c, tx) => handle_create(s, c, tx).await,
        Msg::D(d) => handle_delete(s, d).await,
        Msg::J(j, tx) => handle_join(s, j, tx).await,
        Msg::S(st) => handle_start(s, st).await,
        Msg::B(b) => handle_board(s, b).await,
        Msg::P(p) => handle_participants(s, p).await,
        Msg::Subscribe(tx) => handle_subscribe(s, tx).await,
    };
}

async fn handle_create(s: &mut Session, req: Create, tx: oneshot::Sender<Created>) -> Result {
    if !utils::is_valid_user_name(&req.owner_name) {
        s.rx.close();
        return Err(());
    }
    let auth_token = AuthToken::new();
    s.user_ids.insert(auth_token.clone(), UserId::OWNER);
    s.user_names.insert(UserId::OWNER, req.owner_name);
    let _ = tx.send(Created {
        session_id: s.id.clone(),
        user_id: UserId::OWNER,
        auth_token,
    });
    Ok(())
}

async fn handle_delete(s: &mut Session, req: Delete) -> Result {
    if !s.is_owner(&req.auth_token) {
        return Err(());
    }
    s.rx.close();
    Ok(())
}

async fn handle_join(s: &mut Session, req: Join, tx: oneshot::Sender<Joined>) -> Result {
    match req {
        Join::Join { user_name } => handle_join2(s, user_name, tx).await,
        Join::Rejoin { auth_token } => handle_rejoin(s, auth_token, tx).await,
    }
}

async fn handle_start(s: &mut Session, req: Start) -> Result {
    if s.game.is_some()
        || !s.is_owner(&req.auth_token)
        || s.participants.len() < 4
        || s.participants.len() > MAX_NUM_OF_PARTICIPANTS
    {
        return Err(());
    }
    if s.game_id == 0 {
        let pairings = utils::create_pairings(s.participants.len() as u8);
        s.queue = pairings
            .iter()
            .map(|&((a, b), (c, d))| {
                (
                    (
                        s.participants[(a - 1) as usize],
                        s.participants[(b - 1) as usize],
                    ),
                    (
                        s.participants[(c - 1) as usize],
                        s.participants[(d - 1) as usize],
                    ),
                )
            })
            .collect();
    }
    let active_participants = s.queue.pop_front().ok_or(())?;
    s.reset_game(active_participants);
    s.notify_all();
    Ok(())
}

async fn handle_board(s: &mut Session, req: Board) -> Result {
    match req {
        Board::Deploy {
            auth_token,
            piece,
            pos,
        } => handle_deploy(s, auth_token, piece, pos).await,
        Board::Move { auth_token, change } => handle_move(s, auth_token, change).await,
    }
}

async fn handle_participants(s: &mut Session, req: Participants) -> Result {
    if s.game_id != 0
        || s.game.is_some()
        || !s.is_owner(&req.auth_token)
        || req
            .participants
            .iter()
            .any(|p| s.user_names.get(p).is_none())
    {
        return Err(());
    }
    s.participants = req.participants;
    s.notify_all();
    Ok(())
}

async fn handle_join2(s: &mut Session, user_name: String, tx: oneshot::Sender<Joined>) -> Result {
    if !utils::is_valid_user_name(&user_name) || s.user_ids.len() > MAX_NUM_OF_USERS {
        return Err(());
    }
    let user_id = s.user_ids.len();
    let user_id = UserId::new(user_id as u8);
    let auth_token = AuthToken::new();
    s.user_ids.insert(auth_token.clone(), user_id);
    s.user_names.insert(user_id, user_name.clone());
    let _ = tx.send(Joined {
        user_id,
        user_name,
        auth_token,
    });
    s.notify_all();
    Ok(())
}

async fn handle_rejoin(
    s: &mut Session,
    auth_token: AuthToken,
    tx: oneshot::Sender<Joined>,
) -> Result {
    let user_id = s.user_ids.get(&auth_token).ok_or(())?.clone();
    let user_name = s.user_names.get(&user_id).unwrap().clone();
    let _ = tx.send(Joined {
        user_id,
        user_name,
        auth_token,
    });
    Ok(())
}

async fn handle_deploy(
    s: &mut Session,
    auth_token: AuthToken,
    piece: String,
    pos: String,
) -> Result {
    let user_id = s.user_ids.get(&auth_token).ok_or(())?;
    let game = s.game.as_mut().ok_or(())?;
    let (b1, w) = game.board_and_color(user_id).ok_or(())?;
    let piece = utils::parse_piece(&piece).ok_or(())?;
    let (col, row) = utils::parse_pos(&pos).ok_or(())?;
    if game.logic.deploy_piece(b1, w, piece, row, col).is_err() {
        return Err(());
    }
    s.notify_all();
    s.tick();
    Ok(())
}

async fn handle_move(s: &mut Session, auth_token: AuthToken, change: String) -> Result {
    let game = s.game.as_mut().ok_or(())?;
    let user_id = s.user_ids.get(&auth_token).ok_or(())?;
    let (b1, w) = game.board_and_color(user_id).ok_or(())?;
    let is_whites_turn = game.logic.get_white_active(b1);
    if is_whites_turn != w {
        return Err(());
    }
    let [i, j, i_new, j_new] = utils::parse_change(&change);
    if game.logic.movemaker(b1, i, j, i_new, j_new).is_err() {
        return Err(());
    }
    s.notify_all();
    s.tick();
    Ok(())
}

async fn handle_subscribe(
    s: &mut Session,
    tx: oneshot::Sender<broadcast::Receiver<String>>,
) -> Result {
    let _ = tx.send(s.broadcast_tx.subscribe());
    Ok(())
}
