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

mod serialization;

use std::collections::HashMap;

use hyper::http::response;
use hyper::{body, header, Body, Method, Request, Response, StatusCode};

use crate::common;
use crate::state::StateActor;

use serialization::{Req, Resp};

pub type DispatchResult = Result<Response<Body>, common::Error>;

pub async fn dispatch(state: StateActor, req: Request<Body>) -> DispatchResult {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/events") => dispatch_events(state, req).await,
        (&Method::POST, "/api") => dispatch_api(state, req.into_body()).await,
        _ => not_found(),
    }
}

async fn dispatch_events(mut state: StateActor, req: Request<Body>) -> DispatchResult {
    if let Some(queries) = get_queries(req.uri()) {
        if let Some(&session_id) = queries.get("session_id") {
            let resp = state.subscribe(session_id.to_owned()).await;
            match resp {
                Some(rx) => Ok(event_stream_builder().body(Body::wrap_stream(rx))?),
                _ => bad_request(),
            }
        } else {
            bad_request()
        }
    } else {
        bad_request()
    }
}

async fn dispatch_api(state: StateActor, body: Body) -> DispatchResult {
    let buf = body::to_bytes(body).await?;
    if let Ok(req) = serde_json::from_slice::<Req>(&buf) {
        match req {
            Req::Connect {
                session_id,
                user_name,
            } => dispatch_connect(state, session_id, user_name).await,
            Req::Create { user_name } => dispatch_create(state, user_name).await,
            Req::DeployPiece {
                auth_token,
                piece,
                pos,
            } => dispatch_authenticated_move_deploy(state, auth_token, piece, pos).await,
            Req::MovePiece { auth_token, change } => {
                dispatch_authenticated_move_move(state, auth_token, change).await
            }
            Req::Reconnect { auth_token } => {
                dispatch_authenticated_reconnect(state, auth_token).await
            }
            Req::SetParticipants {
                auth_token,
                participants,
            } => dispatch_authenticated_config_participants(state, auth_token, participants).await,
            Req::Start { auth_token } => {
                dispatch_authenticated_config_start(state, auth_token).await
            }
        }
    } else {
        bad_request()
    }
}

async fn dispatch_connect(
    mut state: StateActor,
    session_id: String,
    user_name: String,
) -> DispatchResult {
    let resp = state.connect(session_id, user_name).await;
    match resp {
        Some((user_id, auth_token)) => Resp::Connected {
            user_id,
            auth_token,
        }
        .into(),
        _ => bad_request(),
    }
}

async fn dispatch_create(mut state: StateActor, user_name: String) -> DispatchResult {
    let resp = state.create(user_name).await;
    match resp {
        Some(auth_token) => Resp::Created { auth_token }.into(),
        _ => bad_request(),
    }
}

async fn dispatch_authenticated_reconnect(
    mut state: StateActor,
    auth_token: String,
) -> DispatchResult {
    let resp = state.reconnect(auth_token).await;
    match resp {
        Some((session_id, user_id, user_name)) => Resp::Reconnected {
            session_id,
            user_id,
            user_name,
        }
        .into(),
        _ => bad_request(),
    }
}

async fn dispatch_authenticated_config_participants(
    mut state: StateActor,
    auth_token: String,
    participants: Vec<String>,
) -> DispatchResult {
    let resp = state.set_participants(auth_token, participants).await;
    match resp {
        Some(()) => ok(),
        _ => bad_request(),
    }
}

async fn dispatch_authenticated_config_start(
    mut state: StateActor,
    auth_token: String,
) -> DispatchResult {
    let resp = state.start(auth_token).await;
    match resp {
        Some(()) => ok(),
        _ => bad_request(),
    }
}

async fn dispatch_authenticated_move_deploy(
    mut state: StateActor,
    auth_token: String,
    piece: String,
    pos: String,
) -> DispatchResult {
    let resp = state.deploy_piece(auth_token, piece, pos).await;
    match resp {
        Some(()) => ok(),
        _ => bad_request(),
    }
}

async fn dispatch_authenticated_move_move(
    mut state: StateActor,
    auth_token: String,
    change: String,
) -> DispatchResult {
    let resp = state.move_piece(auth_token, change).await;
    match resp {
        Some(()) => ok(),
        _ => bad_request(),
    }
}

// Helper functions

fn get_queries(uri: &hyper::Uri) -> Option<HashMap<&str, &str>> {
    Some(
        uri.query()?
            .split('&')
            .map(|q| {
                let mut it = q.splitn(2, '=');
                let k = it.next().unwrap_or("");
                let v = it.next().unwrap_or("");
                (k, v)
            })
            .collect(),
    )
}

// TODO: Don't set Access-Control-Allow-Origin to *
fn builder() -> response::Builder {
    Response::builder().header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
}

fn event_stream_builder() -> response::Builder {
    builder()
        .header(header::CONNECTION, "keep-alive")
        .header(header::CONTENT_TYPE, "text/event-stream")
}

fn json_builder() -> response::Builder {
    builder().header(header::CONTENT_TYPE, "application/json; charset=UTF-8")
}

fn ok() -> DispatchResult {
    Ok(builder().status(StatusCode::OK).body(Body::empty())?)
}

fn bad_request() -> DispatchResult {
    Ok(builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())?)
}

fn unauthorized() -> DispatchResult {
    Ok(builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::empty())?)
}

fn not_found() -> DispatchResult {
    Ok(builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())?)
}

fn internal_server_error() -> DispatchResult {
    Ok(builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())?)
}
