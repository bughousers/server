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

use super::Session;
use crate::common::event::EventType;
use crate::common::req::*;
use crate::common::resp::*;
use crate::common::*;
use futures::channel::oneshot;
use tokio::sync::broadcast;

type Result = std::result::Result<(), ()>;

pub enum Msg {
    C(Create, oneshot::Sender<String>),
    D(Delete),
    J(Join, oneshot::Sender<String>),
    S(Start),
    R(Resign),
    B(Board),
    P(Participants),
    Subscribe(oneshot::Sender<broadcast::Receiver<String>>),
}

pub async fn handle_msg(s: &mut Session, msg: Msg) {
    let _ = match msg {
        Msg::C(c, tx) => handle_create(s, c, tx).await,
        Msg::D(d) => handle_delete(s, d).await,
        Msg::J(j, tx) => handle_join(s, j, tx).await,
        Msg::S(st) => handle_start(s, st).await,
        Msg::R(r) => handle_resign(s, r).await,
        Msg::B(b) => handle_board(s, b).await,
        Msg::P(p) => handle_participants(s, p).await,
        Msg::Subscribe(tx) => handle_subscribe(s, tx).await,
    };
}

pub fn handle_timer(s: &mut Session) {
    s.tick();
}

pub fn handle_broadcast_timer(s: &mut Session) {
    s.notify_all(UserId::OWNER, EventType::Periodic);
}

async fn handle_create(s: &mut Session, req: Create, tx: oneshot::Sender<String>) -> Result {
    let res = s.add_user(req.owner_name);
    if res.is_err() {
        s.rx.close();
        return Err(());
    }
    let (user_id, auth_token) = res.or(Err(()))?;
    let json = serde_json::to_string(&Created {
        session_id: &s.id,
        user_id: &user_id,
        auth_token: &auth_token,
    })
    .unwrap();
    let _ = tx.send(json);
    Ok(())
}

async fn handle_delete(s: &mut Session, req: Delete) -> Result {
    if !s.is_owner(&req.auth_token) {
        return Err(());
    }
    s.rx.close();
    Ok(())
}

async fn handle_join(s: &mut Session, req: Join, tx: oneshot::Sender<String>) -> Result {
    match req {
        Join::Join { user_name } => handle_join2(s, user_name, tx).await,
        Join::Rejoin { auth_token } => handle_rejoin(s, auth_token, tx).await,
    }
}

async fn handle_start(s: &mut Session, req: Start) -> Result {
    if !s.is_owner(&req.auth_token) {
        return Err(());
    }
    s.start_game().or(Err(()))?;
    s.notify_all(UserId::OWNER, EventType::GameStarted);
    Ok(())
}

async fn handle_resign(s: &mut Session, req: Resign) -> Result {
    let user_id = s.user_id(&req.auth_token).ok_or(())?;
    s.game.as_mut().map(|g| g.resign(&user_id));
    s.check_end_conditions();
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
        Board::Promote {
            auth_token,
            change,
            upgrade_to,
        } => handle_promote(s, auth_token, change, upgrade_to).await,
    }
}

async fn handle_participants(s: &mut Session, req: Participants) -> Result {
    if !s.is_owner(&req.auth_token) {
        return Err(());
    }
    s.set_participants(req.participants).or(Err(()))?;
    s.notify_all(UserId::OWNER, EventType::ParticipantsChanged);
    Ok(())
}

async fn handle_join2(s: &mut Session, user_name: String, tx: oneshot::Sender<String>) -> Result {
    let (user_id, auth_token) = s.add_user(user_name).or(Err(()))?;
    let json = serde_json::to_string(&Joined {
        user_id: &user_id,
        auth_token: &auth_token,
        session: s,
    })
    .unwrap();
    let _ = tx.send(json);
    s.notify_all(user_id, EventType::Joined);
    Ok(())
}

async fn handle_rejoin(
    s: &mut Session,
    auth_token: AuthToken,
    tx: oneshot::Sender<String>,
) -> Result {
    let user_id = s.user_ids.get(&auth_token).ok_or(())?;
    let json = serde_json::to_string(&Joined {
        user_id,
        auth_token: &auth_token,
        session: s,
    })
    .unwrap();
    let _ = tx.send(json);
    Ok(())
}

async fn handle_deploy(
    s: &mut Session,
    auth_token: AuthToken,
    piece: String,
    pos: String,
) -> Result {
    let user_id = s.user_id(&auth_token).ok_or(())?;
    let game = s.game.as_mut().ok_or(())?;
    game.deploy_piece(&user_id, &piece, &pos).or(Err(()))?;
    s.check_end_conditions();
    s.notify_all(user_id, EventType::PieceDeployed);
    Ok(())
}

async fn handle_move(s: &mut Session, auth_token: AuthToken, change: String) -> Result {
    let user_id = s.user_id(&auth_token).ok_or(())?;
    let game = s.game.as_mut().ok_or(())?;
    game.move_piece(&user_id, &change).or(Err(()))?;
    s.check_end_conditions();
    s.notify_all(user_id, EventType::PieceMoved);
    Ok(())
}

async fn handle_promote(
    s: &mut Session,
    auth_token: AuthToken,
    change: String,
    upgrade_to: String,
) -> Result {
    let user_id = s.user_id(&auth_token).ok_or(())?;
    let game = s.game.as_mut().ok_or(())?;
    game.promote_piece(&user_id, &change, &upgrade_to)
        .or(Err(()))?;
    s.check_end_conditions();
    s.notify_all(user_id, EventType::PiecePromoted);
    Ok(())
}

async fn handle_subscribe(
    s: &mut Session,
    tx: oneshot::Sender<broadcast::Receiver<String>>,
) -> Result {
    let _ = tx.send(s.broadcast_tx.subscribe());
    Ok(())
}
