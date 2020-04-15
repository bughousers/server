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

use clap::{crate_name, crate_version, App, Arg};
use config::Config;
use dispatcher::dispatch;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use sessions::Sessions;
use std::net::SocketAddr;

mod common;
mod config;
mod dispatcher;
mod session;
mod sessions;

fn parse_args() -> Config {
    let mut config = Config::builder();
    let args = App::new(crate_name!())
        .version(crate_version!())
        .arg(
            Arg::with_name("bind")
                .long("bind")
                .short("b")
                .takes_value(true)
                .value_name("ADDR"),
        )
        .get_matches();
    if let Some(addr) = args.value_of("bind") {
        config = config.bind_addr::<SocketAddr>(addr.parse().unwrap());
    }
    config.build()
}

#[tokio::main]
async fn main() -> Result<(), hyper::Error> {
    let config = parse_args();
    let sessions = Sessions::new();
    sessions.garbage_collect().await;
    let make_svc = make_service_fn(|_| {
        let sessions = sessions.clone();
        async { Ok::<_, hyper::Error>(service_fn(move |req| dispatch(sessions.clone(), req))) }
    });
    Server::bind(config.bind_addr()).serve(make_svc).await
}
