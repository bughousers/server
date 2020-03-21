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
use crate::registry::Message as RegistryMessage;
use crate::session;
use hyper::http::response;
use hyper::{body, header, Body, StatusCode};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum Message {
    Response(Response),
    Error(MessageError),
}

#[derive(Debug)]
pub enum MessageError {
    AuthTokenInvalid,
    CannotParse,
    MustBeSessionOwner,
    PreconditionFailure,
    SessionIdInvalid,
    TooManyUsers,
    UserNameInvalid,
}

impl Into<hyper::Response<Body>> for MessageError {
    fn into(self) -> hyper::Response<Body> {
        builder()
            .status(match self {
                MessageError::AuthTokenInvalid => StatusCode::UNAUTHORIZED,
                MessageError::CannotParse => StatusCode::BAD_REQUEST,
                MessageError::MustBeSessionOwner => StatusCode::FORBIDDEN,
                MessageError::PreconditionFailure => StatusCode::BAD_REQUEST,
                MessageError::SessionIdInvalid => StatusCode::NOT_FOUND,
                MessageError::TooManyUsers => StatusCode::BAD_REQUEST,
                MessageError::UserNameInvalid => StatusCode::BAD_REQUEST,
            })
            .body(hyper::Body::empty())
            .unwrap()
    }
}

pub type Result = std::result::Result<hyper::Response<Body>, Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

type Sender = mpsc::Sender<RegistryMessage>;

pub async fn dispatch(mut handle: Sender, req: hyper::Request<Body>) -> Result {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/events") => dispatch_events(handle, req).await,
        (&hyper::Method::POST, "/api") => dispatch_api(handle, req.into_body()).await,
        _ => not_found(),
    }
}

async fn dispatch_events(mut handle: Sender, req: hyper::Request<Body>) -> Result {
    if let Some(queries) = get_queries(req.uri()) {
        if let Some(&session_id) = queries.get("session_id") {
            let (tx, rx) = oneshot::channel();
            if handle
                .send(RegistryMessage::Relay(
                    session_id.into(),
                    session::Message::Subscribe(tx),
                ))
                .await
                .is_err()
            {
                return internal_server_error();
            }
            match rx.await {
                Ok(rx) => Ok(event_stream_builder().body(Body::wrap_stream(rx))?),
                _ => not_found(),
            }
        } else {
            bad_request()
        }
    } else {
        bad_request()
    }
}

async fn dispatch_api(mut handle: Sender, body: Body) -> Result {
    let buf = body::to_bytes(body).await?;
    if let Ok(req) = serde_json::from_slice::<Request>(&buf) {
        let (tx, rx) = oneshot::channel();
        if handle
            .send(RegistryMessage::Request(req, tx))
            .await
            .is_err()
        {
            return internal_server_error();
        }
        match rx.await {
            Ok(Message::Response(resp)) => Ok(json_builder().body(resp.into())?),
            Ok(Message::Error(err)) => Ok(err.into()),
            _ => internal_server_error(),
        }
    } else {
        bad_request()
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
    hyper::Response::builder().header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
}

fn event_stream_builder() -> response::Builder {
    builder()
        .header(header::CONNECTION, "keep-alive")
        .header(header::CONTENT_TYPE, "text/event-stream")
}

fn json_builder() -> response::Builder {
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
