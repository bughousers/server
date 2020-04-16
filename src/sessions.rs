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

use crate::{
    common::*,
    config::Config,
    session::{Msg, Session},
};
use futures::channel::mpsc;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

const GC_INTERVAL: Duration = Duration::from_secs(900);

#[derive(Clone)]
pub struct Sessions {
    inner: Arc<Inner>,
}

struct Inner {
    sessions: RwLock<HashMap<SessionId, mpsc::Sender<Msg>>>,
    config: Arc<Config>,
}

impl Inner {
    fn new(config: Arc<Config>) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            config,
        }
    }
}

impl Sessions {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            inner: Arc::new(Inner::new(config)),
        }
    }

    pub async fn get(&self, id: &SessionId) -> Option<mpsc::Sender<Msg>> {
        self.inner.sessions.read().await.get(id).cloned()
    }

    pub async fn spawn(&self, owner_name: &str) -> Option<mpsc::Sender<Msg>> {
        let session_id = SessionId::new();
        let (session, tx) =
            Session::new(self.inner.config.clone(), session_id.clone(), owner_name)?;
        session.spawn();
        self.inner
            .sessions
            .write()
            .await
            .insert(session_id, tx.clone());
        Some(tx)
    }

    pub async fn garbage_collect(&self) {
        let s = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::delay_for(GC_INTERVAL).await;
                let mut marked: Vec<SessionId> = Vec::with_capacity(0);
                let sessions = &mut s.inner.sessions.write().await;
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
