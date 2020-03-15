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

use hyper::{Body, Method, Request, Response};
use serde::{Deserialize, Serialize};

pub type DispatchError = Box<dyn Error + Send + Sync>;
pub type DispatchResult = Result<Response<Body>, DispatchError>;

pub async fn dispatch(req: Request<Body>) -> DispatchResult {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/create") => dispatch_create(req).await,
        _ => Ok(Response::new("dispatch()".into())),
    }
}

#[derive(Deserialize, Serialize)]
struct CreateResp {}

async fn dispatch_create(req: Request<Body>) -> DispatchResult {
    Ok(Response::new("dispatch_create()".into()))
}
