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

use hyper::http::response::Builder;
use hyper::{Body, Method, Request, Response};
use serde::{Deserialize, Serialize};

pub type DispatchError = Box<dyn Error + Send + Sync>;
pub type DispatchResult = Result<Response<Body>, DispatchError>;

pub async fn dispatch(req: Request<Body>) -> DispatchResult {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/events") => dispatch_events(req).await,
        (&Method::POST, "/connect") => dispatch_connect(req).await,
        (&Method::POST, "/create") => dispatch_create(req).await,
        (&Method::POST, "/reconnect") => dispatch_reconnect(req).await,
        (&Method::POST, "/update") => dispatch_update(req).await,
        _ => not_found(),
    }
}

// Handle /events requests

#[derive(Deserialize, Serialize)]
struct Event {}

async fn dispatch_events(req: Request<Body>) -> DispatchResult {
    Ok(Response::new("dispatch_events()".into()))
}

// Handle /connect requests

#[derive(Deserialize, Serialize)]
struct ConnectReq {}

#[derive(Deserialize, Serialize)]
struct ConnectResp {}

async fn dispatch_connect(req: Request<Body>) -> DispatchResult {
    Ok(Response::new("dispatch_connect()".into()))
}

// Handle /create requests

#[derive(Deserialize, Serialize)]
struct CreateResp {}

async fn dispatch_create(req: Request<Body>) -> DispatchResult {
    Ok(Response::new("dispatch_create()".into()))
}

// Handle /reconnect requests

#[derive(Deserialize, Serialize)]
struct ReconnectReq {}

#[derive(Deserialize, Serialize)]
struct ReconnectResp {}

async fn dispatch_reconnect(req: Request<Body>) -> DispatchResult {
    Ok(Response::new("dispatch_reconnect()".into()))
}

// Handle /update requests

#[derive(Deserialize, Serialize)]
struct UpdateReq {}

#[derive(Deserialize, Serialize)]
struct UpdateResp {}

async fn dispatch_update(req: Request<Body>) -> DispatchResult {
    Ok(Response::new("dispatch_update()".into()))
}

// Helper functions

// TODO: Don't set Access-Control-Allow-Origin to *
fn builder() -> Builder {
    Response::builder().header("Access-Control-Allow-Origin", "*")
}

fn not_found() -> DispatchResult {
    Ok(builder().status(404).body(Body::empty())?)
}
