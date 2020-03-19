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
mod state;

use std::error::Error;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use dispatcher::dispatch;
use state::StateActor;

pub type ServerError = Box<dyn Error + Send + Sync>;

const LISTEN_ADDR: &'static str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let state = StateActor::new();
    let state_copy = state.clone();
    tokio::spawn(async move {
        let mut state = state_copy;
        loop {
            tokio::time::delay_for(std::time::Duration::from_secs(900)).await;
            state.garbage_collect().await;
        }
    });
    let make_svc = make_service_fn(|_| {
        let state = state.clone();
        async { Ok::<_, ServerError>(service_fn(move |req| dispatch(state.clone(), req))) }
    });
    Server::bind(&LISTEN_ADDR.parse()?).serve(make_svc).await?;
    Ok(())
}
