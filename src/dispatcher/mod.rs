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
#[cfg(test)]
mod tests;

use hyper::body;
use hyper::http::response::Builder;
use hyper::{Body, Method, Request, Response};
use tokio::sync::mpsc::channel;

use super::state::{Channel, Msg, MsgData, MsgResp};
use super::ServerError;
use serialization::{AuthorizedReq, ConfigReq, MoveReq, Req, Resp};

pub type DispatchResult = Result<Response<Body>, ServerError>;

pub async fn dispatch(ch: Channel, req: Request<Body>) -> DispatchResult {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/events") => dispatch_events(req).await,
        (&Method::POST, "/api") => dispatch_api(ch, req.into_body()).await,
        _ => not_found(),
    }
}

async fn dispatch_events(_: Request<Body>) -> DispatchResult {
    Ok(Response::new("dispatch_events()".into()))
}

async fn dispatch_api(ch: Channel, body: Body) -> DispatchResult {
    let buf = body::to_bytes(body).await?;
    if let Ok(req) = serde_json::from_slice::<Req>(&buf) {
        match req {
            Req::Connect {
                sessionId,
                userName,
            } => dispatch_connect(ch, sessionId, userName).await,
            Req::Create { userName } => dispatch_create(ch, userName).await,
            Req::Authorized {
                userId,
                authToken,
                req,
            } => dispatch_authorized(ch, userId, authToken, req).await,
        }
    } else {
        bad_request()
    }
}

async fn dispatch_connect(ch: Channel, session_id: String, user_name: String) -> DispatchResult {
    if !validate_user_name(&user_name) {
        return bad_request();
    }
    let (tx, mut rx) = channel::<MsgResp>(1);
    let msg = Msg {
        data: MsgData::Connect(session_id.into(), user_name),
        resp_channel: tx,
    };
    if let Err(_) = ch.send(msg) {
        return internal_server_error();
    }
    match rx.recv().await {
        Some(MsgResp::Connected(uid, tok)) => Ok(json_builder().body(
            serde_json::to_string(&Resp::Connected {
                userId: uid.into(),
                authToken: tok.into(),
            })?
            .into(),
        )?),
        Some(MsgResp::ConnectFailure) => not_found(),
        _ => internal_server_error(),
    }
}

async fn dispatch_create(ch: Channel, user_name: String) -> DispatchResult {
    if !validate_user_name(&user_name) {
        return bad_request();
    }
    let (tx, mut rx) = channel::<MsgResp>(1);
    let msg = Msg {
        data: MsgData::Create(user_name),
        resp_channel: tx,
    };
    if let Err(_) = ch.send(msg) {
        return internal_server_error();
    }
    let msg_resp = rx.recv().await;
    if let Some(MsgResp::Created(sid, uid, tok)) = msg_resp {
        Ok(json_builder().body(
            serde_json::to_string(&Resp::Created {
                sessionId: sid.into(),
                userId: uid.into(),
                authToken: tok.into(),
            })?
            .into(),
        )?)
    } else {
        internal_server_error()
    }
}

async fn dispatch_authorized(
    ch: Channel,
    user_id: String,
    auth_token: String,
    req: AuthorizedReq,
) -> DispatchResult {
    match req {
        AuthorizedReq::Config { req } => dispatch_config(ch, user_id, auth_token, req).await,
        AuthorizedReq::Move { req } => dispatch_move(ch, user_id, auth_token, req).await,
        AuthorizedReq::Reconnect => dispatch_reconnect(ch, user_id, auth_token).await,
    }
}

async fn dispatch_config(
    ch: Channel,
    user_id: String,
    auth_token: String,
    req: ConfigReq,
) -> DispatchResult {
    let (tx, mut rx) = channel::<MsgResp>(1);
    let msg = match req {
        ConfigReq::Participants { participants } => Msg {
            data: MsgData::ChangeParticipants(
                user_id.into(),
                auth_token.into(),
                participants.iter().map(|p| p.clone().into()).collect(),
            ),
            resp_channel: tx,
        },
        ConfigReq::Start => Msg {
            data: MsgData::Start(user_id.into(), auth_token.into()),
            resp_channel: tx,
        },
    };
    if let Err(_) = ch.send(msg) {
        return internal_server_error();
    }
    match rx.recv().await {
        Some(MsgResp::ChangedParticipants) => ok(),
        Some(MsgResp::ChangeParticipantsFailure) => unauthorized(),
        Some(MsgResp::Started) => ok(),
        Some(MsgResp::StartFailure) => unauthorized(),
        _ => internal_server_error(),
    }
}

async fn dispatch_move(
    ch: Channel,
    user_id: String,
    auth_token: String,
    req: MoveReq,
) -> DispatchResult {
    let (tx, mut rx) = channel::<MsgResp>(1);
    let msg = match req {
        MoveReq::Move { change } => Msg {
            data: MsgData::Move(user_id.into(), auth_token.into(), change),
            resp_channel: tx,
        },
    };
    if let Err(_) = ch.send(msg) {
        return internal_server_error();
    }
    match rx.recv().await {
        Some(MsgResp::Moved) => ok(),
        Some(MsgResp::MoveFailure) => unauthorized(),
        _ => internal_server_error(),
    }
}

async fn dispatch_reconnect(ch: Channel, user_id: String, auth_token: String) -> DispatchResult {
    let (tx, mut rx) = channel::<MsgResp>(1);
    let msg = Msg {
        data: MsgData::Reconnect(user_id.into(), auth_token.into()),
        resp_channel: tx,
    };
    if let Err(_) = ch.send(msg) {
        return internal_server_error();
    }
    match rx.recv().await {
        Some(MsgResp::Reconnected(sid, n)) => Ok(json_builder().body(
            serde_json::to_string(&Resp::Reconnected {
                sessionId: sid.into(),
                userName: n,
            })?
            .into(),
        )?),
        Some(MsgResp::ReconnectFailure) => unauthorized(),
        _ => internal_server_error(),
    }
}

// Helper functions

fn validate_user_name(name: &String) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}

// TODO: Don't set Access-Control-Allow-Origin to *
fn builder() -> Builder {
    Response::builder().header("Access-Control-Allow-Origin", "*")
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
