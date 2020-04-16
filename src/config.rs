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

    pub fn debug(self, value: bool) -> Self {
        Self {
            config: Config {
                debug: value,
                ..self.config
            },
        }
    }

    pub fn threads(self, value: usize) -> Self {
        Self {
            config: Config {
                threads: value,
                ..self.config
            },
        }
    }

    pub fn bind_addr<T: Into<SocketAddr>>(self, value: T) -> Self {
        Self {
            config: Config {
                bind_addr: value.into(),
                ..self.config
            },
        }
    }

    pub fn max_session(self, value: usize) -> Self {
        Self {
            config: Config {
                max_session: value,
                ..self.config
            },
        }
    }

    pub fn session_capacity(self, value: usize) -> Self {
        Self {
            config: Config {
                session_capacity: value,
                ..self.config
            },
        }
    }

    pub fn tick(self, value: Duration) -> Self {
        Self {
            config: Config {
                tick: value,
                ..self.config
            },
        }
    }

    pub fn broadcast_interval(self, value: Duration) -> Self {
        Self {
            config: Config {
                broadcast_interval: value,
                ..self.config
            },
        }
    }

    pub fn max_user(self, value: usize) -> Self {
        Self {
            config: Config {
                max_user: value,
                ..self.config
            },
        }
    }

    pub fn max_participant(self, value: usize) -> Self {
        Self {
            config: Config {
                max_participant: value,
                ..self.config
            },
        }
    }
}
