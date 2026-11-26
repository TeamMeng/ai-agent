use std::collections::{HashMap, hash_map::Entry};

use crate::session::model::Session;

#[async_trait::async_trait]
pub trait SessionManager {
    async fn create(&mut self, session_id: &str, user_id: Option<&str>) -> anyhow::Result<Session>;
    async fn get(&self, session_id: &str) -> anyhow::Result<Option<Session>>;
    async fn save(&mut self, session: Session) -> anyhow::Result<()>;
    async fn get_or_create(
        &mut self,
        session_id: &str,
        user_id: Option<&str>,
    ) -> anyhow::Result<Session>;
}

pub struct InMemorySessionManager {
    pub sessions: HashMap<String, Session>,
}

#[async_trait::async_trait]
impl SessionManager for InMemorySessionManager {
    async fn create(&mut self, session_id: &str, user_id: Option<&str>) -> anyhow::Result<Session> {
        if self.sessions.contains_key(session_id) {
            anyhow::bail!("Session already exists: {}", session_id)
        }

        let session = Session::new(session_id.to_owned(), user_id.map(str::to_owned));
        match self.sessions.entry(session_id.to_owned()) {
            Entry::Occupied(_) => {
                anyhow::bail!("Session already exists: {}", session_id)
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(session.clone());
                Ok(session)
            }
        }
    }

    async fn get(&self, session_id: &str) -> anyhow::Result<Option<Session>> {
        let one = self.sessions.get(session_id).cloned();
        Ok(one)
    }

    async fn save(&mut self, session: Session) -> anyhow::Result<()> {
        self.sessions.insert(session.session_id.clone(), session);
        Ok(())
    }

    async fn get_or_create(
        &mut self,
        session_id: &str,
        user_id: Option<&str>,
    ) -> anyhow::Result<Session> {
        if let Some(session) = self.get(session_id).await? {
            return Ok(session);
        }

        self.create(session_id, user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_then_get_returns_the_session() {
        let mut manager = InMemorySessionManager {
            sessions: HashMap::new(),
        };

        let created = manager.create("s1", Some("u1")).await.expect("create");
        let fetched = manager.get("s1").await.expect("get");

        let fetched = fetched.expect("session should exist");
        assert_eq!(fetched.session_id, created.session_id);
        assert_eq!(fetched.user_id.as_deref(), Some("u1"));
    }

    #[tokio::test]
    async fn creating_a_duplicate_session_fails() {
        let mut manager = InMemorySessionManager {
            sessions: HashMap::new(),
        };

        manager.create("s1", None).await.expect("first create");

        assert!(manager.create("s1", None).await.is_err());
    }

    #[tokio::test]
    async fn get_or_create_reuses_existing_and_creates_missing() {
        let mut manager = InMemorySessionManager {
            sessions: HashMap::new(),
        };

        let first = manager
            .get_or_create("s1", Some("u1"))
            .await
            .expect("first get_or_create");
        let second = manager
            .get_or_create("s1", Some("u1"))
            .await
            .expect("second get_or_create");
        assert_eq!(first.session_id, second.session_id);

        let created = manager.get_or_create("s2", None).await.expect("create s2");
        assert_eq!(created.session_id, "s2");
        assert_eq!(manager.sessions.len(), 2);
    }
}
