#[cfg(test)]
mod tests {
    use super::AuthService;
    use crate::repositories::{UserRepo, UserRecord};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;
    use anyhow::Result;

    struct InMemoryUserRepo {
        inner: Mutex<HashMap<String, UserRecord>>,
    }

    impl InMemoryUserRepo {
        fn new() -> Self {
            Self { inner: Mutex::new(HashMap::new()) }
        }
    }

    #[async_trait]
    impl UserRepo for InMemoryUserRepo {
        async fn create_user(&self, id: Uuid, email: &str, password_hash: &str, role: &str) -> anyhow::Result<()> {
            let mut guard = self.inner.lock().unwrap();
            guard.insert(email.to_string(), UserRecord { id, email: email.to_string(), password_hash: password_hash.to_string(), role: role.to_string() });
            Ok(())
        }

        async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<UserRecord>> {
            let guard = self.inner.lock().unwrap();
            Ok(guard.get(email).cloned())
        }
    }

    #[tokio::test]
    async fn test_register_and_login_and_verify() -> Result<()> {
        let repo = InMemoryUserRepo::new();
        let arc_repo: std::sync::Arc<dyn UserRepo> = std::sync::Arc::new(repo);
        let auth = AuthService::new(arc_repo, "test-secret".into(), 3600);

        let id = auth.register("unit@test", "password").await?;
        assert!(!id.to_string().is_empty());

        let token = auth.login("unit@test", "password").await?;
        assert!(!token.is_empty());

        let claims = auth.verify_token(&token)?;
        assert_eq!(claims.sub, id.to_string());

        Ok(())
    }

    #[tokio::test]
    async fn test_login_invalid() -> Result<()> {
        let repo = InMemoryUserRepo::new();
        let arc_repo: std::sync::Arc<dyn UserRepo> = std::sync::Arc::new(repo);
        let auth = AuthService::new(arc_repo, "test-secret".into(), 3600);

        let res = auth.login("missing@test", "password").await;
        assert!(res.is_err());
        Ok(())
    }
}
