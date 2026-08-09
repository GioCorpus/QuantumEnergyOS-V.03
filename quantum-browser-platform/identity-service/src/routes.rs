use axum::{routing::{post, get}, Router, extract::State, Json, http::StatusCode, middleware, response::IntoResponse, Extension};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{NewUser, UserPublic, LoginRequest};
use argon2::{Argon2, password_hash::{SaltString, PasswordHasher, PasswordVerifier, PasswordHash}, rand_core::OsRng};
use jsonwebtoken::{encode, EncodingKey, Header, decode, DecodingKey, Validation};
use std::env;
use http::{Request, StatusCode as HttpStatusCode};
use axum::middleware::Next;
use serde::{Serialize, Deserialize as SerdeDeserialize};

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(ErrorResponse { code: self.status.as_u16(), message: self.message });
        (self.status, body).into_response()
    }
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }
}

use crate::AppState;

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

async fn register(State(app): State<AppState>, Json(payload): Json<RegisterReq>) -> Result<impl IntoResponse, ApiError> {
    let id = match app.auth.register(&payload.email, &payload.password).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("register error: {}", e);
            return Err(ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "could not create user"));
        }
    };
    let public = UserPublic { id: id.to_string(), email: payload.email };
    Ok((StatusCode::CREATED, Json(public)))
}

async fn login(State(app): State<AppState>, Json(payload): Json<LoginRequest>) -> Result<impl IntoResponse, ApiError> {
    match app.auth.login(&payload.email, &payload.password).await {
        Ok(t) => Ok((StatusCode::OK, Json(serde_json::json!({"token": t})))),
        Err(_) => Err(ApiError::new(StatusCode::UNAUTHORIZED, "invalid credentials")),
    }
}

use std::sync::Arc;
use crate::services::auth::AuthService;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub sub: String,
}

#[derive(SerdeDeserialize, Debug, Serialize)]
struct Claims {
    sub: String,
    exp: i64,
}

async fn auth_middleware<B>(auth: Arc<AuthService>, req: Request<B>, next: Next<B>) -> Result<impl IntoResponse, ApiError> {
    // Check Authorization header
    let auth_header = req.headers().get("authorization").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    if auth_header.is_none() {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "missing authorization header"));
    }
    let auth_header = auth_header.unwrap();
    if !auth_header.to_lowercase().starts_with("bearer ") {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "invalid authorization header"));
    }
    let token = auth_header[7..].trim();

    match auth.verify_token(token) {
        Ok(claims) => {
            let mut req = req;
            req.extensions_mut().insert(AuthUser { sub: claims.sub });
            Ok(next.run(req).await)
        }
        Err(e) => {
            tracing::warn!("token verify failed: {}", e);
            Err(ApiError::new(StatusCode::UNAUTHORIZED, "invalid token"))
        }
    }
}

async fn me(Extension(user): Extension<AuthUser>) -> Result<impl IntoResponse, ApiError> {
    Ok((StatusCode::OK, Json(serde_json::json!({"user": {"id": user.sub}}))))
}
