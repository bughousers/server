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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            debug: false,
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
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

    pub fn debug(self, debug: bool) -> Self {
        Self {
            config: Config {
                debug,
                ..self.config
            },
        }
    }

    pub fn bind_addr<T: Into<SocketAddr>>(self, t: T) -> Self {
        Self {
            config: Config {
                bind_addr: t.into(),
                ..self.config
            },
        }
    }
}
