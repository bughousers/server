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

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct Config {
    debug: bool,
    threads: usize,
    bind_addr: SocketAddr,
    max_session: usize,
    session_capacity: usize,
    tick: Duration,
    broadcast_interval: Duration,
    max_user: usize,
    max_participant: usize,
}

impl Config {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn bind_addr(&self) -> &SocketAddr {
        &self.bind_addr
    }

    pub fn max_session(&self) -> usize {
        self.max_session
    }

    pub fn session_capacity(&self) -> usize {
        self.session_capacity
    }

    pub fn tick(&self) -> Duration {
        self.tick
    }

    pub fn broadcast_interval(&self) -> Duration {
        self.broadcast_interval
    }

    pub fn max_user(&self) -> usize {
        self.max_user
    }

    pub fn max_participant(&self) -> usize {
        self.max_participant
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            debug: false,
            threads: 2,
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            max_session: 10,
            session_capacity: 4,
            tick: Duration::from_secs(2),
            broadcast_interval: Duration::from_secs(20),
            max_user: 20,
            max_participant: 5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Builder {
    config: Config,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn build(self) -> Config {
        self.config
    }

    pub fn debug(&mut self, value: bool) -> &mut Self {
        self.config.debug = value;
        self
    }

    pub fn threads(&mut self, value: usize) -> &mut Self {
        self.config.threads = value;
        self
    }

    pub fn bind_addr<T: Into<SocketAddr>>(&mut self, value: T) -> &mut Self {
        self.config.bind_addr = value.into();
        self
    }

    pub fn max_session(&mut self, value: usize) -> &mut Self {
        self.config.max_session = value;
        self
    }

    pub fn session_capacity(&mut self, value: usize) -> &mut Self {
        self.config.session_capacity = value;
        self
    }

    pub fn tick(&mut self, value: Duration) -> &mut Self {
        self.config.tick = value;
        self
    }

    pub fn broadcast_interval(&mut self, value: Duration) -> &mut Self {
        self.config.broadcast_interval = value;
        self
    }

    pub fn max_user(&mut self, value: usize) -> &mut Self {
        self.config.max_user = value;
        self
    }

    pub fn max_participant(&mut self, value: usize) -> &mut Self {
        self.config.max_participant = value;
        self
    }
}
