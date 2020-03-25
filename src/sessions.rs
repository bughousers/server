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
use crate::session::{Msg, Session};
use futures::channel::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

const GC_INTERVAL: Duration = Duration::from_secs(900);

#[derive(Clone)]
pub struct Sessions {
    inner: Arc<RwLock<SessionsInner>>,
}

struct SessionsInner {
    sessions: HashMap<SessionId, mpsc::Sender<Msg>>,
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

    pub async fn get(&self, id: &SessionId) -> Option<mpsc::Sender<Msg>> {
        self.inner.read().await.sessions.get(id).cloned()
    }

    pub async fn spawn(&self, owner_name: &str) -> Option<mpsc::Sender<Msg>> {
        let session_id = SessionId::new();
        let (session, tx) = Session::new(session_id.clone(), owner_name)?;
        session.spawn();
        self.inner
            .write()
            .await
            .sessions
            .insert(session_id, tx.clone());
        Some(tx)
    }

    pub async fn garbage_collect(&self) {
        let s = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::delay_for(GC_INTERVAL).await;
                let mut marked: Vec<SessionId> = Vec::with_capacity(0);
                let sessions = &mut s.inner.write().await.sessions;
                for (sid, s) in sessions.iter() {
                    if s.is_closed() {
                        marked.push(sid.clone());
                    }
                }
                for sid in marked {
                    sessions.remove(&sid);
                }
            }
        });
    }
}
