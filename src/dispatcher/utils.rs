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

use hyper::{
    header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONNECTION, CONTENT_TYPE},
    http::response::Builder,
    Body, Response, StatusCode,
};

// TODO: Don't set Access-Control-Allow-Origin to *
pub fn builder() -> Builder {
    Response::builder().header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
}

pub fn event_stream_builder() -> Builder {
    builder()
        .header(CONNECTION, "keep-alive")
        .header(CONTENT_TYPE, "text/event-stream")
}

pub fn json_builder() -> Builder {
    builder().header(CONTENT_TYPE, "application/json; charset=UTF-8")
}

pub fn to_json<T: Into<Body>>(t: T) -> Response<Body> {
    json_builder().body(t.into()).unwrap()
}

pub fn accepted() -> Response<Body> {
    builder()
        .status(StatusCode::ACCEPTED)
        .body(Body::empty())
        .unwrap()
}

pub fn bad_request() -> Response<Body> {
    builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())
        .unwrap()
}

pub fn not_found() -> Response<Body> {
    builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}
