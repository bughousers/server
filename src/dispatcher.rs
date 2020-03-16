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

use std::error::Error;

use hyper::body;
use hyper::http::response::Builder;
use hyper::{Body, Method, Request, Response};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::channel;

use crate::state::state::{Channel, Msg, MsgData, MsgResp};

pub type DispatchError = Box<dyn Error + Send + Sync>;
pub type DispatchResult = Result<Response<Body>, DispatchError>;

pub async fn dispatch(ch: Channel, req: Request<Body>) -> DispatchResult {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/events") => dispatch_events(req).await,
        (&Method::POST, "/connect") => dispatch_connect(req).await,
        (&Method::POST, "/create") => dispatch_create(ch, req.into_body()).await,
        (&Method::POST, "/reconnect") => dispatch_reconnect(ch, req.into_body()).await,
        (&Method::POST, "/update") => dispatch_update(req).await,
        _ => not_found(),
    }
}

// Handle /events requests

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct Event {}

async fn dispatch_events(req: Request<Body>) -> DispatchResult {
    Ok(Response::new("dispatch_events()".into()))
}

// Handle /connect requests

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct ConnectReq {}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct ConnectResp {}

async fn dispatch_connect(req: Request<Body>) -> DispatchResult {
    Ok(Response::new("dispatch_connect()".into()))
}

// Handle /create requests

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct CreateReq {
    userName: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct CreateResp {
    sessionId: String,
    userId: String,
    authToken: String,
}

async fn dispatch_create(ch: Channel, body: Body) -> DispatchResult {
    let data = body::to_bytes(body).await?;
    if let Ok(req) = serde_json::from_slice::<CreateReq>(&data) {
        if !validate_user_name(&req.userName) {
            return bad_request();
        }
        let (tx, mut rx) = channel::<MsgResp>(1);
        let msg = Msg {
            data: MsgData::Create(req.userName),
            resp_channel: tx,
        };
        if let Err(_) = ch.send(msg) {
            return internal_server_error();
        }
        let msg_resp = rx.recv().await;
        if let Some(MsgResp::Created(sid, uid, tok)) = msg_resp {
            Ok(json_builder().body(
                serde_json::to_string(&CreateResp {
                    sessionId: sid.into(),
                    userId: uid.into(),
                    authToken: tok.into(),
                })?
                .into(),
            )?)
        } else {
            internal_server_error()
        }
    } else {
        bad_request()
    }
}

// Handle /reconnect requests

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct ReconnectReq {
    sessionId: String,
    userId: String,
    authToken: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct ReconnectResp {
    userName: String,
}

async fn dispatch_reconnect(ch: Channel, body: Body) -> DispatchResult {
    let data = body::to_bytes(body).await?;
    if let Ok(req) = serde_json::from_slice::<ReconnectReq>(&data) {
        let (tx, mut rx) = channel::<MsgResp>(1);
        let msg = Msg {
            data: MsgData::Reconnect(
                req.sessionId.into(),
                req.userId.into(),
                req.authToken.into(),
            ),
            resp_channel: tx,
        };
        if let Err(_) = ch.send(msg) {
            return internal_server_error();
        }
        match rx.recv().await {
            Some(MsgResp::Reconnected(n)) => Ok(json_builder()
                .body(serde_json::to_string(&ReconnectResp { userName: n })?.into())?),
            Some(MsgResp::ReconnectAuthFailure) => unauthorized(),
            Some(MsgResp::ReconnectFailure) => not_found(),
            _ => internal_server_error(),
        }
    } else {
        bad_request()
    }
}

// Handle /update requests

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct UpdateReq {}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct UpdateResp {}

async fn dispatch_update(req: Request<Body>) -> DispatchResult {
    Ok(Response::new("dispatch_update()".into()))
}

// Helper functions

fn validate_user_name(name: &String) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
}

// TODO: Don't set Access-Control-Allow-Origin to *
fn builder() -> Builder {
    Response::builder().header("Access-Control-Allow-Origin", "*")
}

fn json_builder() -> Builder {
    builder().header("Content-Type", "application/json; charset=UTF-8")
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
