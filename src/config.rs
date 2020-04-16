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

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Clone, Debug)]
pub struct Config {
    debug: bool,
    bind_addr: SocketAddr,
    session_capacity: usize,
    max_user: usize,
}

impl Config {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    pub fn bind_addr(&self) -> &SocketAddr {
        &self.bind_addr
    }

    pub fn session_capacity(&self) -> usize {
        self.session_capacity
    }

    pub fn max_user(&self) -> usize {
        self.max_user
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            debug: false,
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            session_capacity: 4,
            max_user: 20,
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

    pub fn bind_addr<T: Into<SocketAddr>>(self, value: T) -> Self {
        Self {
            config: Config {
                bind_addr: value.into(),
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

    pub fn max_user(self, value: usize) -> Self {
        Self {
            config: Config {
                max_user: value,
                ..self.config
            },
        }
    }
}
