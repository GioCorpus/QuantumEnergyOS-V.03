use async_trait::async_trait;
use uuid::Uuid;
use sqlx::PgPool;

pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
}

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn create_user(&self, id: Uuid, email: &str, password_hash: &str, role: &str) -> anyhow::Result<()>;
    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<UserRecord>>;
}

pub mod postgres {
    use super::*;
    use sqlx::Row;

    pub struct PostgresUserRepo {
        pub pool: PgPool,
    }

    impl PostgresUserRepo {
        pub fn new(pool: PgPool) -> Self {
            Self { pool }
        }
    }

    #[async_trait]
    impl UserRepo for PostgresUserRepo {
        async fn create_user(&self, id: Uuid, email: &str, password_hash: &str, role: &str) -> anyhow::Result<()> {
            sqlx::query!(
                r#"INSERT INTO users (id, email, password_hash, role) VALUES ($1, $2, $3, $4)"#,
                id,
                email,
                password_hash,
                role
            )
            .execute(&self.pool)
            .await?;
            Ok(())
        }

        async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<UserRecord>> {
            let row = sqlx::query!("SELECT id, email, password_hash, role FROM users WHERE email = $1", email)
                .fetch_optional(&self.pool)
                .await?;
            if let Some(r) = row {
                Ok(Some(UserRecord { id: r.id, email: r.email, password_hash: r.password_hash, role: r.role }))
            } else {
                Ok(None)
            }
        }
    }
}
