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
use std::{net::SocketAddr, sync::Arc};
use tokio::runtime;

mod common;
mod config;
mod data;
mod dispatcher;
mod session;
mod sessions;

fn parse_args() -> Config {
    let mut builder = Config::builder();
    let args = App::new(crate_name!())
        .version(crate_version!())
        .arg(Arg::with_name("debug").long("debug").short("d"))
        .arg(
            Arg::with_name("threads")
                .long("threads")
                .short("t")
                .takes_value(true)
                .value_name("NUM"),
        )
        .arg(
            Arg::with_name("bind")
                .long("bind")
                .short("b")
                .takes_value(true)
                .value_name("ADDR"),
        )
        .get_matches();
    if args.is_present("debug") {
        builder.debug(true);
    }
    if let Some(num) = args.value_of("threads") {
        builder.threads(num.parse::<usize>().unwrap());
    }
    if let Some(addr) = args.value_of("bind") {
        builder.bind_addr(addr.parse::<SocketAddr>().unwrap());
    }
    builder.build()
}

fn main() {
    let config = Arc::new(parse_args());
    if config.debug() {
        println!("Current configuration: {:?}", config);
    }
    let mut rt = runtime::Builder::new();
    if config.threads() > 1 {
        rt.threaded_scheduler().core_threads(config.threads());
    } else {
        rt.basic_scheduler();
    }
    let mut rt = rt.enable_all().build().unwrap();
    if config.debug() {
        println!("Using Tokio runtime: {:?}", rt);
    }
    let sessions = Sessions::new(config.clone());
    let make_svc = make_service_fn(|_| {
        let sessions = sessions.clone();
        async { Ok::<_, hyper::Error>(service_fn(move |req| dispatch(sessions.clone(), req))) }
    });
    let _ = rt.block_on(async { Server::bind(config.bind_addr()).serve(make_svc).await });
}
