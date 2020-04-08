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

mod utils;
mod v1;

use crate::sessions::Sessions;
use crate::LISTEN_ADDR;
use hyper::Body;
use url::Url;
use utils::not_found;

type Request = hyper::Request<Body>;

pub type Result = std::result::Result<Response, BoxError>;
pub type Response = hyper::Response<Body>;
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub async fn dispatch(sessions: Sessions, req: Request) -> Result {
    let url = format!("http://{}{}", LISTEN_ADDR, req.uri());
    let url = Url::parse(&url).unwrap();
    let parts: Vec<&str> = url.path_segments().unwrap().collect();
    match parts.split_first() {
        Some((&"v1", rest)) => v1::dispatch(sessions, rest, req).await,
        _ => not_found(),
    }
}
