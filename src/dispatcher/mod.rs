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

// TODO: Make this private again. Events should
// be handled inside the dispatcher.
pub mod serialization;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use hyper::body;
use hyper::http::response::Builder;
use hyper::{Body, Method, Request, Response};
use tokio::sync::mpsc::channel;

use super::state::SessionId;
use super::state::{Channel, Msg, MsgData, MsgResp};
use super::ServerError;
use serialization::{AuthenticatedReq, ConfigReq, MoveReq, Req, Resp};

pub type DispatchResult = Result<Response<Body>, ServerError>;

pub async fn dispatch(ch: Channel, req: Request<Body>) -> DispatchResult {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/events") => dispatch_events(ch, req).await,
        (&Method::POST, "/api") => dispatch_api(ch, req.into_body()).await,
        _ => not_found(),
    }
}

async fn dispatch_events(mut ch: Channel, req: Request<Body>) -> DispatchResult {
    if let Some(query) = req.uri().query() {
        let queries: HashMap<&str, &str> = query
            .split('&')
            .map(|q| {
                let mut it = q.splitn(2, '=');
                let k = it.next().unwrap_or("");
                let v = it.next().unwrap_or("");
                (k, v)
            })
            .collect();
        if let Some(&session_id) = queries.get("session_id") {
            let resp = msg(&mut ch, MsgData::Subscribe(session_id.to_owned().into())).await;
            match resp {
                Some(MsgResp::Subscribed(rx)) => {
                    Ok(event_stream_builder().body(Body::wrap_stream(rx))?)
                }
                _ => internal_server_error(),
            }
        } else {
            not_found()
        }
    } else {
        not_found()
    }
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
            Req::Authenticated {
                userId,
                authToken,
                req,
            } => dispatch_authenticated(ch, userId, authToken, req).await,
        }
    } else {
        bad_request()
    }
}

async fn dispatch_connect(
    mut ch: Channel,
    session_id: String,
    user_name: String,
) -> DispatchResult {
    if !validate_user_name(&user_name) {
        return bad_request();
    }
    let resp = msg(&mut ch, MsgData::Connect(session_id.into(), user_name)).await;
    match resp {
        Some(MsgResp::Connected(uid, tok)) => Ok(json_builder().body(
            Resp::Connected {
                userId: uid.into(),
                authToken: tok.into(),
            }
            .into(),
        )?),
        Some(MsgResp::ConnectFailure) => not_found(),
        _ => internal_server_error(),
    }
}

async fn dispatch_create(mut ch: Channel, user_name: String) -> DispatchResult {
    if !validate_user_name(&user_name) {
        return bad_request();
    }
    let resp = msg(&mut ch, MsgData::Create(user_name)).await;
    if let Some(MsgResp::Created(sid, uid, tok)) = resp {
        Ok(json_builder().body(
            Resp::Created {
                sessionId: sid.into(),
                userId: uid.into(),
                authToken: tok.into(),
            }
            .into(),
        )?)
    } else {
        internal_server_error()
    }
}

async fn dispatch_authenticated(
    mut ch: Channel,
    user_id: String,
    auth_token: String,
    req: AuthenticatedReq,
) -> DispatchResult {
    let resp = msg(
        &mut ch,
        MsgData::Authenticate(user_id.clone().into(), auth_token.clone().into()),
    )
    .await;
    match resp {
        Some(MsgResp::Authenticated(sid)) => match req {
            AuthenticatedReq::Config { req } => dispatch_config(ch, sid, user_id, req).await,
            AuthenticatedReq::Move { req } => dispatch_move(ch, sid, user_id, req).await,
            AuthenticatedReq::Reconnect => dispatch_reconnect(ch, sid, user_id).await,
        },
        Some(MsgResp::AuthenticateFailure) => unauthorized(),
        _ => internal_server_error(),
    }
}

async fn dispatch_config(
    mut ch: Channel,
    session_id: SessionId,
    user_id: String,
    req: ConfigReq,
) -> DispatchResult {
    let data = match req {
        ConfigReq::Participants { participants } => MsgData::ChangeParticipants(
            session_id,
            user_id.into(),
            participants.iter().map(|p| p.clone().into()).collect(),
        ),
        ConfigReq::Start => MsgData::Start(session_id, user_id.into()),
    };
    let resp = msg(&mut ch, data).await;
    match resp {
        Some(MsgResp::ChangedParticipants) => ok(),
        Some(MsgResp::ChangeParticipantsFailure) => unauthorized(),
        Some(MsgResp::Started) => ok(),
        Some(MsgResp::StartFailure) => unauthorized(),
        _ => internal_server_error(),
    }
}

async fn dispatch_move(
    mut ch: Channel,
    session_id: SessionId,
    user_id: String,
    req: MoveReq,
) -> DispatchResult {
    let data = match req {
        MoveReq::Deploy { piece, pos } => MsgData::Deploy(session_id, user_id.into(), piece, pos),
        MoveReq::Move { change } => MsgData::Move(session_id, user_id.into(), change),
    };
    let resp = msg(&mut ch, data).await;
    match resp {
        Some(MsgResp::Moved) => ok(),
        Some(MsgResp::MoveFailure) => unauthorized(),
        _ => internal_server_error(),
    }
}

async fn dispatch_reconnect(
    mut ch: Channel,
    session_id: SessionId,
    user_id: String,
) -> DispatchResult {
    let resp = msg(&mut ch, MsgData::Reconnect(session_id, user_id.into())).await;
    match resp {
        Some(MsgResp::Reconnected(sid, n)) => Ok(json_builder().body(
            Resp::Reconnected {
                sessionId: sid.into(),
                userName: n,
            }
            .into(),
        )?),
        Some(MsgResp::ReconnectFailure) => unauthorized(),
        _ => internal_server_error(),
    }
}

// Helper functions

async fn msg(ch: &mut Channel, data: MsgData) -> Option<MsgResp> {
    let (tx, mut rx) = channel::<MsgResp>(1);
    let msg = Msg {
        data: data,
        resp_channel: tx,
    };
    ch.send(msg).ok()?;
    rx.recv().await
}

fn validate_user_name(name: &String) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
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
