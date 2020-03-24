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

mod common;
mod dispatcher;
mod session;
mod sessions;

use dispatcher::dispatch;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Result, Server};
use sessions::Sessions;
use std::time::Duration;

const GC_INTERVAL: Duration = Duration::from_secs(900);
pub const LISTEN_ADDR: &'static str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> Result<()> {
    let sessions = Sessions::new();
    let sessions2 = sessions.clone();
    tokio::spawn(async move {
        tokio::time::delay_for(GC_INTERVAL).await;
        sessions2.garbage_collect().await;
    });
    let make_svc = make_service_fn(|_| {
        let sessions = sessions.clone();
        async { Ok::<_, Error>(service_fn(move |req| dispatch(sessions.clone(), req))) }
    });
    Server::bind(&LISTEN_ADDR.parse().unwrap())
        .serve(make_svc)
        .await
        .unwrap();
    Ok(())
}
