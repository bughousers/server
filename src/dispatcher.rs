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
use crate::sessions::Sessions;
use hyper::http::response::Builder;
use hyper::{body, header, Body, Method, Response, StatusCode};
use url::Url;

type Request = hyper::Request<Body>;
pub type Result = std::result::Result<Response<Body>, Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn dispatch(sessions: Sessions, req: Request) -> Result {
    let url = Url::parse(&req.uri().to_string())?;
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
        not_found() // TODO: Implement
    } else if let Some((&sid, rest)) = parts.split_first() {
        dispatch_session(sessions, rest, req, sid).await
    } else {
        not_found()
    }
}

async fn dispatch_session(
    sessions: Sessions,
    parts: &[&str],
    req: Request,
    session_id: &str,
) -> Result {
    if parts.is_empty() {
        if req.method() == &Method::POST {
            not_found() // TODO: Implement
        } else if req.method() == &Method::DELETE {
            not_found() // TODO: Implement
        } else {
            not_found()
        }
    } else {
        match parts.split_first() {
            Some((&"games", rest)) => dispatch_games(sessions, rest, req, session_id).await,
            Some((&"participants", rest)) => {
                dispatch_participants(sessions, rest, req, session_id).await
            }
            Some((&"sse", rest)) => dispatch_sse(sessions, rest, req, session_id).await,
            _ => not_found(),
        }
    }
}

async fn dispatch_games(
    sessions: Sessions,
    parts: &[&str],
    req: Request,
    session_id: &str,
) -> Result {
    if parts.is_empty() && req.method() == &Method::POST {
        not_found() // TODO: Implement
    } else if let Some((&gid, rest)) = parts.split_first() {
        dispatch_game(sessions, rest, req, session_id, gid).await
    } else {
        not_found()
    }
}

async fn dispatch_participants(
    sessions: Sessions,
    parts: &[&str],
    req: Request,
    session_id: &str,
) -> Result {
    if parts.is_empty() && req.method() == &Method::PUT {
        not_found() // TODO: Implement
    } else {
        not_found()
    }
}

async fn dispatch_sse(
    sessions: Sessions,
    parts: &[&str],
    req: Request,
    session_id: &str,
) -> Result {
    if parts.is_empty() && req.method() == &Method::GET {
        not_found() // TODO: Implement
    } else {
        not_found()
    }
}

async fn dispatch_game(
    sessions: Sessions,
    parts: &[&str],
    req: Request,
    session_id: &str,
    game_id: &str,
) -> Result {
    match parts.split_first() {
        Some((&"board", rest)) => dispatch_board(sessions, rest, req, session_id, game_id).await,
        _ => not_found(),
    }
}

async fn dispatch_board(
    sessions: Sessions,
    parts: &[&str],
    req: Request,
    session_id: &str,
    game_id: &str,
) -> Result {
    if parts.is_empty() && req.method() == &Method::POST {
        not_found() // TODO: Implement
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

fn bad_request() -> Result {
    Ok(builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())?)
}

fn not_found() -> Result {
    Ok(builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())?)
}

fn internal_server_error() -> Result {
    Ok(builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())?)
}
