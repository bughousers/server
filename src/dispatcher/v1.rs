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

use super::{error::Error, utils::*, Request, Result};
use crate::{common::*, session::Msg, sessions::Sessions};
use futures::{
    channel::{mpsc, oneshot},
    SinkExt,
};
use hyper::{body, Body, Method};

pub async fn dispatch(sessions: Sessions, parts: &[&str], req: Request) -> Result {
    match parts.split_first() {
        Some((&"sessions", rest)) => dispatch_sessions(sessions, rest, req).await,
        _ => Err(Error::InvalidResource),
    }
}

async fn dispatch_sessions(sessions: Sessions, parts: &[&str], req: Request) -> Result {
    if parts.is_empty() && req.method() == &Method::POST {
        let json = body::to_bytes(req.into_body()).await?;
        let req = serde_json::from_slice::<req::Create>(&json)?;
        let mut session = sessions
            .spawn(&req.owner_name)
            .await
            .ok_or(Error::InvalidRequest)?;
        let (tx, rx) = oneshot::channel();
        session.send(Msg::C(req, tx)).await?;
        Ok(to_json(rx.await?))
    } else if let Some((&sid, rest)) = parts.split_first() {
        let session = sessions
            .get(&sid.into())
            .await
            .ok_or(Error::InvalidResource)?;
        dispatch_session(session, rest, req).await
    } else {
        Err(Error::InvalidResource)
    }
}

async fn dispatch_session(mut session: mpsc::Sender<Msg>, parts: &[&str], req: Request) -> Result {
    match (parts, req.method()) {
        ([], &Method::DELETE) => {
            let json = body::to_bytes(req.into_body()).await?;
            let req = serde_json::from_slice::<req::Delete>(&json)?;
            session.send(Msg::D(req)).await?;
            Ok(accepted())
        }
        ([], &Method::POST) => {
            let json = body::to_bytes(req.into_body()).await?;
            let req = serde_json::from_slice::<req::Join>(&json)?;
            let (tx, rx) = oneshot::channel();
            session.send(Msg::J(req, tx)).await?;
            Ok(to_json(rx.await?))
        }
        (["games"], &Method::POST) => {
            let json = body::to_bytes(req.into_body()).await?;
            let req = serde_json::from_slice::<req::Start>(&json)?;
            session.send(Msg::S(req)).await?;
            Ok(accepted())
        }
        (["games", _], &Method::POST) => {
            let json = body::to_bytes(req.into_body()).await?;
            let req = serde_json::from_slice::<req::Resign>(&json)?;
            session.send(Msg::R(req)).await?;
            Ok(accepted())
        }
        (["games", _, "board"], &Method::POST) => {
            let json = body::to_bytes(req.into_body()).await?;
            let req = serde_json::from_slice::<req::Board>(&json)?;
            session.send(Msg::B(req)).await?;
            Ok(accepted())
        }
        (["participants"], &Method::POST) => {
            let json = body::to_bytes(req.into_body()).await?;
            let req = serde_json::from_slice::<req::Participants>(&json)?;
            session.send(Msg::P(req)).await?;
            Ok(accepted())
        }
        (["sse"], &Method::GET) => {
            let (tx, rx) = oneshot::channel();
            session.send(Msg::Subscribe(tx)).await?;
            let rx = rx.await?;
            Ok(event_stream_builder().body(Body::wrap_stream(rx)).unwrap())
        }
        _ => Err(Error::InvalidResource),
    }
}
