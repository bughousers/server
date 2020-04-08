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

mod error;
mod utils;
mod v1;

use crate::sessions::Sessions;
use error::Error;
use hyper::{Body, Response};
use url::Url;
use utils::{bad_request, not_found};

type Request = hyper::Request<Body>;

type Result = StdResult<Response<Body>, Error>;
type StdResult<T, E> = std::result::Result<T, E>;

pub async fn dispatch(sessions: Sessions, req: Request) -> StdResult<Response<Body>, hyper::Error> {
    match real_dispatch(sessions, req).await {
        Ok(resp) => Ok(resp),
        Err(Error::Hyper(err)) => Err(err),
        Err(Error::InvalidRequest) => Ok(bad_request()),
        Err(Error::InvalidResource) => Ok(not_found()),
    }
}

async fn real_dispatch(sessions: Sessions, req: Request) -> Result {
    let uri = req.uri();
    let scheme = uri.scheme().ok_or(Error::InvalidRequest)?;
    let authority = uri.authority().ok_or(Error::InvalidRequest)?;
    let path_and_query = uri.path_and_query().ok_or(Error::InvalidRequest)?;
    let url = format!("{}://{}{}", scheme, authority, path_and_query);
    let url = Url::parse(&url)?;
    let parts: Vec<&str> = url.path_segments().unwrap().collect();
    match parts.split_first() {
        Some((&"v1", rest)) => v1::dispatch(sessions, rest, req).await,
        _ => Err(Error::InvalidResource),
    }
}
