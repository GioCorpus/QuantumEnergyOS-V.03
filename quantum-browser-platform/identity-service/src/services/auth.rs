use crate::repositories::UserRecord;
use crate::repositories::postgres::PostgresUserRepo;
use crate::repositories::UserRepo;
use argon2::{Argon2, password_hash::{SaltString, PasswordHasher, PasswordVerifier, PasswordHash}, rand_core::OsRng};
use uuid::Uuid;
use jsonwebtoken::{encode, EncodingKey, Header, decode, DecodingKey, Validation};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub sub: String,
    pub exp: i64,
}

pub struct AuthService {
    pub repo: Arc<dyn UserRepo>,
    pub jwt_secret: String,
    pub token_exp_seconds: i64,
}

impl AuthService {
    pub fn new(repo: Arc<dyn UserRepo>, jwt_secret: String, token_exp_seconds: i64) -> Self {
        Self { repo, jwt_secret, token_exp_seconds }
    }

    pub async fn register(&self, email: &str, password: &str) -> anyhow::Result<Uuid> {
        let salt = SaltString::generate(&mut OsRng);
        let argon = Argon2::default();
        let hash = argon.hash_password(password.as_bytes(), &salt)?.to_string();
        let id = Uuid::new_v4();
        self.repo.create_user(id, email, &hash, "user").await?;
        Ok(id)
    }

    pub async fn login(&self, email: &str, password: &str) -> anyhow::Result<String> {
        if let Some(user) = self.repo.find_by_email(email).await? {
            let ph = PasswordHash::new(&user.password_hash)?;
            Argon2::default().verify_password(password.as_bytes(), &ph)?;
            let claims = Claims { sub: user.id.to_string(), exp: chrono::Utc::now().timestamp() + self.token_exp_seconds };
            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(self.jwt_secret.as_bytes()))?;
            Ok(token)
        } else {
            anyhow::bail!("invalid credentials")
        }
    }

    pub fn verify_token(&self, token: &str) -> anyhow::Result<Claims> {
        let data = decode::<Claims>(token, &DecodingKey::from_secret(self.jwt_secret.as_bytes()), &Validation::default())?;
        Ok(data.claims)
    }
}
