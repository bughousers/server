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

use hyper::body;
use hyper::http::response::Builder;
use hyper::{Body, Method, Request, Response};

use crate::common;
use crate::session;
use crate::state::{AuthenticatedMsg, Channel, Msg, Reply, State};

use serialization::{Authenticated, Config, Move, Req, Resp};

pub type DispatchResult = Result<Response<Body>, common::Error>;

pub async fn dispatch(ch: Channel, req: Request<Body>) -> DispatchResult {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/events") => dispatch_events(ch, req).await,
        (&Method::POST, "/api") => dispatch_api(ch, req.into_body()).await,
        _ => not_found(),
    }
}

async fn dispatch_events(mut ch: Channel, req: Request<Body>) -> DispatchResult {
    if let Some(queries) = get_queries(req.uri()) {
        if let Some(&session_id) = queries.get("session_id") {
            let resp = State::msg(
                &mut ch,
                Msg::Subscribe {
                    session_id: session_id.to_owned(),
                },
            )
            .await;
            match resp {
                Some(Reply::Subscribe { rx }) => {
                    Ok(event_stream_builder().body(Body::wrap_stream(rx))?)
                }
                Some(Reply::Failure) => bad_request(),
                _ => internal_server_error(),
            }
        } else {
            bad_request()
        }
    } else {
        bad_request()
    }
}

async fn dispatch_api(ch: Channel, body: Body) -> DispatchResult {
    let buf = body::to_bytes(body).await?;
    if let Ok(req) = serde_json::from_slice::<Req>(&buf) {
        match req {
            Req::Authenticated { auth_token, data } => {
                dispatch_authenticated(ch, auth_token, data).await
            }
            Req::Connect {
                session_id,
                user_name,
            } => dispatch_connect(ch, session_id, user_name).await,
            Req::Create { user_name } => dispatch_create(ch, user_name).await,
        }
    } else {
        bad_request()
    }
}

async fn dispatch_authenticated(
    ch: Channel,
    auth_token: String,
    data: Authenticated,
) -> DispatchResult {
    match data {
        Authenticated::Config { data } => dispatch_authenticated_config(ch, auth_token, data).await,
        Authenticated::Move { data } => dispatch_authenticated_move(ch, auth_token, data).await,
        Authenticated::Reconnect => dispatch_authenticated_reconnect(ch, auth_token).await,
    }
}

async fn dispatch_connect(
    mut ch: Channel,
    session_id: String,
    user_name: String,
) -> DispatchResult {
    let resp = State::msg(
        &mut ch,
        Msg::Connect {
            session_id,
            user_name,
        },
    )
    .await;
    match resp {
        Some(Reply::Connect {
            user_id,
            auth_token,
        }) => Resp::Connected {
            user_id,
            auth_token,
        }
        .into(),
        Some(Reply::Failure) => bad_request(),
        _ => internal_server_error(),
    }
}

async fn dispatch_create(mut ch: Channel, user_name: String) -> DispatchResult {
    let resp = State::msg(&mut ch, Msg::Create { user_name }).await;
    match resp {
        Some(Reply::Create { auth_token }) => Resp::Created { auth_token }.into(),
        Some(Reply::Failure) => bad_request(),
        _ => internal_server_error(),
    }
}

async fn dispatch_authenticated_config(
    ch: Channel,
    auth_token: String,
    data: Config,
) -> DispatchResult {
    match data {
        Config::Participants { participants } => {
            dispatch_authenticated_config_participants(ch, auth_token, participants).await
        }
        Config::Start => dispatch_authenticated_config_start(ch, auth_token).await,
    }
}

async fn dispatch_authenticated_move(
    ch: Channel,
    auth_token: String,
    data: Move,
) -> DispatchResult {
    match data {
        Move::Deploy { piece, pos } => {
            dispatch_authenticated_move_deploy(ch, auth_token, piece, pos).await
        }
        Move::Move { change } => dispatch_authenticated_move_move(ch, auth_token, change).await,
    }
}

async fn dispatch_authenticated_reconnect(mut ch: Channel, auth_token: String) -> DispatchResult {
    let resp = State::msg(
        &mut ch,
        Msg::Authenticated {
            auth_token,
            msg: AuthenticatedMsg::Reconnect,
        },
    )
    .await;
    match resp {
        Some(Reply::Reconnect {
            session_id,
            user_id,
            user_name,
        }) => Resp::Reconnected {
            session_id,
            user_id,
            user_name,
        }
        .into(),
        Some(Reply::Failure) => bad_request(),
        _ => internal_server_error(),
    }
}

async fn dispatch_authenticated_config_participants(
    mut ch: Channel,
    auth_token: String,
    participants: Vec<String>,
) -> DispatchResult {
    let resp = State::msg(
        &mut ch,
        Msg::Authenticated {
            auth_token: auth_token.clone(),
            msg: AuthenticatedMsg::Relay {
                msg: session::Msg::Authorized {
                    auth_token,
                    msg: session::AuthorizedMsg::SetParticipants {
                        user_ids: participants,
                    },
                },
            },
        },
    )
    .await;
    match resp {
        Some(Reply::Success) => ok(),
        Some(Reply::Relay {
            reply: session::Reply::Failure,
        }) => bad_request(),
        _ => internal_server_error(),
    }
}

async fn dispatch_authenticated_config_start(
    mut ch: Channel,
    auth_token: String,
) -> DispatchResult {
    let resp = State::msg(
        &mut ch,
        Msg::Authenticated {
            auth_token: auth_token.clone(),
            msg: AuthenticatedMsg::Relay {
                msg: session::Msg::Authorized {
                    auth_token,
                    msg: session::AuthorizedMsg::Start,
                },
            },
        },
    )
    .await;
    match resp {
        Some(Reply::Success) => ok(),
        Some(Reply::Relay {
            reply: session::Reply::Failure,
        }) => bad_request(),
        _ => internal_server_error(),
    }
}

async fn dispatch_authenticated_move_deploy(
    mut ch: Channel,
    auth_token: String,
    piece: String,
    pos: String,
) -> DispatchResult {
    let resp = State::msg(
        &mut ch,
        Msg::Authenticated {
            auth_token: auth_token.clone(),
            msg: AuthenticatedMsg::Relay {
                msg: session::Msg::DeployPiece {
                    auth_token,
                    piece,
                    pos,
                },
            },
        },
    )
    .await;
    match resp {
        Some(Reply::Success) => ok(),
        Some(Reply::Relay {
            reply: session::Reply::Failure,
        }) => bad_request(),
        _ => internal_server_error(),
    }
}

async fn dispatch_authenticated_move_move(
    mut ch: Channel,
    auth_token: String,
    change: String,
) -> DispatchResult {
    let resp = State::msg(
        &mut ch,
        Msg::Authenticated {
            auth_token: auth_token.clone(),
            msg: AuthenticatedMsg::Relay {
                msg: session::Msg::MovePiece { auth_token, change },
            },
        },
    )
    .await;
    match resp {
        Some(Reply::Success) => ok(),
        Some(Reply::Relay {
            reply: session::Reply::Failure,
        }) => bad_request(),
        _ => internal_server_error(),
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
fn builder() -> Builder {
    Response::builder().header("Access-Control-Allow-Origin", "*")
}

fn event_stream_builder() -> Builder {
    builder()
        .header("Connection", "keep-alive")
        .header("Content-Type", "text/event-stream")
}

fn json_builder() -> Builder {
    builder().header("Content-Type", "application/json; charset=UTF-8")
}

fn ok() -> DispatchResult {
    Ok(builder().status(200).body(Body::empty())?)
}

fn bad_request() -> DispatchResult {
    Ok(builder().status(400).body(Body::empty())?)
}

fn unauthorized() -> DispatchResult {
    Ok(builder().status(401).body(Body::empty())?)
}

fn not_found() -> DispatchResult {
    Ok(builder().status(404).body(Body::empty())?)
}

fn internal_server_error() -> DispatchResult {
    Ok(builder().status(500).body(Body::empty())?)
}
