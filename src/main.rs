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

mod dispatcher;
mod state;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use dispatcher::{dispatch, DispatchError};
use state::State;

const LISTEN_ADDR: &'static str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> Result<(), DispatchError> {
    let tx = State::new().serve();
    let make_svc = make_service_fn(|_| {
        let tx = tx.clone();
        async { Ok::<_, DispatchError>(service_fn(move |req| dispatch(tx.clone(), req))) }
    });
    Server::bind(&LISTEN_ADDR.parse()?).serve(make_svc).await?;
    Ok(())
}
