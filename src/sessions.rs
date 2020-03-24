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
use crate::session::Session;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Sessions {
    inner: Arc<RwLock<SessionsInner>>,
}

struct SessionsInner {
    sessions: HashMap<SessionId, Session>,
}

impl SessionsInner {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(SessionsInner::new())),
        }
    }

    pub async fn get(&self, id: &SessionId) -> Option<Session> {
        self.inner.read().await.sessions.get(id).cloned()
    }

    pub async fn insert(&self, id: SessionId, session: Session) {
        self.inner.write().await.sessions.insert(id, session);
    }

    pub async fn remove(&self, id: &SessionId) {
        self.inner.write().await.sessions.remove(id);
    }

    pub async fn garbage_collect(&self) {
        let mut marked: Vec<SessionId> = Vec::with_capacity(0);
        let sessions = &mut self.inner.write().await.sessions;
        for (sid, s) in sessions.iter() {
            if !s.with(|s| s.is_alive()).await {
                marked.push(sid.clone());
            }
        }
        for sid in marked {
            sessions.remove(&sid);
        }
    }
}
