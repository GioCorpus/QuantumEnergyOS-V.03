use axum::{routing::{post, get}, Router, extract::State, Json, http::StatusCode};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{NewUser, UserPublic, LoginRequest};
use argon2::{Argon2, password_hash::{SaltString, PasswordHasher, PasswordVerifier, PasswordHash}, rand_core::OsRng};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::env;

pub fn auth_routes() -> Router<PgPool> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

pub fn user_routes() -> Router<PgPool> {
    Router::new()
        .route("/me", get(me))
}

#[derive(Deserialize)]
struct RegisterReq {
    email: String,
    password: String,
}

async fn register(State(pool): State<PgPool>, Json(payload): Json<RegisterReq>) -> (StatusCode, Json<UserPublic>) {
    let hash = {
        let salt = SaltString::generate(&mut OsRng);
        let argon = Argon2::default();
        argon.hash_password(payload.password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .unwrap_or_default()
    };

    let id = Uuid::new_v4();
    let role = "user";

    let res = sqlx::query!(
        r#"INSERT INTO users (id, email, password_hash, role) VALUES ($1, $2, $3, $4)"#,
        id,
        payload.email,
        hash,
        role
    )
    .execute(&pool)
    .await;

    match res {
        Ok(_) => {
            let public = UserPublic { id: id.to_string(), email: payload.email };
            (StatusCode::CREATED, Json(public))
        }
        Err(e) => {
            tracing::error!("register error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(UserPublic { id: "".into(), email: "".into() }))
        }
    }
}

async fn login(State(pool): State<PgPool>, Json(payload): Json<LoginRequest>) -> (StatusCode, Json<serde_json::Value>) {
    // fetch user by email
    let rec = sqlx::query!("SELECT id, password_hash FROM users WHERE email = $1", payload.email)
        .fetch_one(&pool)
        .await;

    if let Err(e) = rec {
        tracing::warn!("login lookup failed: {}", e);
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"invalid credentials"})));    }

    let row = rec.unwrap();
    let ph = PasswordHash::new(&row.password_hash).map_err(|e| tracing::error!("hash parse: {}", e)).ok();
    if ph.is_none() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"invalid credentials"})));    }
    let ph = ph.unwrap();

    let v = Argon2::default().verify_password(payload.password.as_bytes(), &ph);
    if v.is_err() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"invalid credentials"})));    }

    // create JWT
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
    let claims = serde_json::json!({"sub": row.id.to_string(), "exp": (chrono::Utc::now().timestamp() + 3600)});
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).map_err(|e| tracing::error!("jwt err: {}", e)).ok();

    if let Some(t) = token {
        (StatusCode::OK, Json(serde_json::json!({"token": t})))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error":"could not create token"})))
    }
}

async fn me() -> (StatusCode, Json<serde_json::Value>) {
    // placeholder, will require auth middleware later
    (StatusCode::OK, Json(serde_json::json!({"user":"stub"})))
}
