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

use crate::common::*;
use crate::session::Msg;
use crate::sessions::Sessions;
use crate::LISTEN_ADDR;
use futures::channel::{mpsc, oneshot};
use futures::SinkExt;
use hyper::http::response::Builder;
use hyper::{body, header, Body, Method, Response, StatusCode};
use url::Url;

pub type Result = std::result::Result<Response<Body>, BoxError>;
type Request = hyper::Request<Body>;
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub async fn dispatch(sessions: Sessions, req: Request) -> Result {
    let url = format!("http://{}{}", LISTEN_ADDR, req.uri());
    let url = Url::parse(&url).unwrap();
    let parts: Vec<&str> = url.path_segments().unwrap().collect();
    match parts.split_first() {
        Some((&"v1", rest)) => dispatch_v1(sessions, rest, req).await,
        _ => not_found(),
    }
}

async fn dispatch_v1(sessions: Sessions, parts: &[&str], req: Request) -> Result {
    match parts.split_first() {
        Some((&"sessions", rest)) => dispatch_sessions(sessions, rest, req).await,
        _ => not_found(),
    }
}

async fn dispatch_sessions(sessions: Sessions, parts: &[&str], req: Request) -> Result {
    if parts.is_empty() && req.method() == &Method::POST {
        let json = body::to_bytes(req.into_body()).await?;
        let req = serde_json::from_slice::<req::Create>(&json)?;
        if let Some(mut session) = sessions.spawn(&req.owner_name).await {
            let (tx, rx) = oneshot::channel();
            session.send(Msg::C(req, tx)).await?;
            to_json(rx.await?)
        } else {
            bad_request()
        }
    } else if let Some((&sid, rest)) = parts.split_first() {
        if let Some(session) = sessions.get(&sid.into()).await {
            dispatch_session(session, rest, req).await
        } else {
            not_found()
        }
    } else {
        not_found()
    }
}

async fn dispatch_session(mut session: mpsc::Sender<Msg>, parts: &[&str], req: Request) -> Result {
    if parts.is_empty() {
        if req.method() == &Method::POST {
            let json = body::to_bytes(req.into_body()).await?;
            let req = serde_json::from_slice::<req::Join>(&json)?;
            let (tx, rx) = oneshot::channel();
            session.send(Msg::J(req, tx)).await?;
            to_json(rx.await?)
        } else if req.method() == &Method::DELETE {
            let json = body::to_bytes(req.into_body()).await?;
            let req = serde_json::from_slice::<req::Delete>(&json)?;
            session.send(Msg::D(req)).await?;
            accepted()
        } else {
            not_found()
        }
    } else {
        match parts.split_first() {
            Some((&"games", rest)) => dispatch_games(session, rest, req).await,
            Some((&"participants", rest)) => dispatch_participants(session, rest, req).await,
            Some((&"sse", rest)) => dispatch_sse(session, rest, req).await,
            _ => not_found(),
        }
    }
}

async fn dispatch_games(mut session: mpsc::Sender<Msg>, parts: &[&str], req: Request) -> Result {
    if parts.is_empty() && req.method() == &Method::POST {
        let json = body::to_bytes(req.into_body()).await?;
        let req = serde_json::from_slice::<req::Start>(&json)?;
        session.send(Msg::S(req)).await?;
        accepted()
    } else if let Some((&gid, rest)) = parts.split_first() {
        dispatch_game(session, rest, req, gid).await
    } else {
        not_found()
    }
}

async fn dispatch_participants(
    mut session: mpsc::Sender<Msg>,
    parts: &[&str],
    req: Request,
) -> Result {
    if parts.is_empty() && req.method() == &Method::PUT {
        let json = body::to_bytes(req.into_body()).await?;
        let req = serde_json::from_slice::<req::Participants>(&json)?;
        session.send(Msg::P(req)).await?;
        accepted()
    } else {
        not_found()
    }
}

async fn dispatch_sse(mut session: mpsc::Sender<Msg>, parts: &[&str], req: Request) -> Result {
    if parts.is_empty() && req.method() == &Method::GET {
        let (tx, rx) = oneshot::channel();
        session.send(Msg::Subscribe(tx)).await?;
        let rx = rx.await?;
        Ok(event_stream_builder().body(Body::wrap_stream(rx)).unwrap())
    } else {
        not_found()
    }
}

async fn dispatch_game(
    mut session: mpsc::Sender<Msg>,
    parts: &[&str],
    req: Request,
    game_id: &str,
) -> Result {
    if parts.is_empty() && req.method() == &Method::POST {
        let json = body::to_bytes(req.into_body()).await?;
        let req = serde_json::from_slice::<req::Resign>(&json)?;
        session.send(Msg::R(req)).await?;
        accepted()
    } else {
        match parts.split_first() {
            Some((&"board", rest)) => dispatch_board(session, rest, req, game_id).await,
            _ => not_found(),
        }
    }
}

async fn dispatch_board(
    mut session: mpsc::Sender<Msg>,
    parts: &[&str],
    req: Request,
    game_id: &str,
) -> Result {
    if parts.is_empty() && req.method() == &Method::POST {
        let json = body::to_bytes(req.into_body()).await?;
        let req = serde_json::from_slice::<req::Board>(&json)?;
        session.send(Msg::B(req)).await?;
        accepted()
    } else {
        not_found()
    }
}

// Helper functions

// TODO: Don't set Access-Control-Allow-Origin to *
fn builder() -> Builder {
    Response::builder().header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
}

fn event_stream_builder() -> Builder {
    builder()
        .header(header::CONNECTION, "keep-alive")
        .header(header::CONTENT_TYPE, "text/event-stream")
}

fn json_builder() -> Builder {
    builder().header(header::CONTENT_TYPE, "application/json; charset=UTF-8")
}

fn to_json<T: Into<Body>>(t: T) -> Result {
    Ok(json_builder().body(t.into()).unwrap())
}

fn accepted() -> Result {
    Ok(builder()
        .status(StatusCode::ACCEPTED)
        .body(Body::empty())
        .unwrap())
}

fn bad_request() -> Result {
    Ok(builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())
        .unwrap())
}

fn not_found() -> Result {
    Ok(builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap())
}
