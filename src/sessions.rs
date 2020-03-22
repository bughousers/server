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

use crate::common::*;
use crate::session::Message;
use futures::channel::mpsc::Sender;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// `Sessions` maps `SessionId` to a channel which can be used to communicate
/// with the intended `Session`.
#[derive(Clone)]
pub struct Sessions {
    sessions: Arc<RwLock<HashMap<SessionId, Sender<Message>>>>,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, id: SessionId, session: Sender<Message>) {
        self.sessions.write().await.insert(id, session);
    }

    pub async fn get(&self, id: &SessionId) -> Option<Sender<Message>> {
        // We clone the channel so that the read lock can be released
        // immediately.
        self.sessions.read().await.get(id).map(|s| s.clone())
    }
}
