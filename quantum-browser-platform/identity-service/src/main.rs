use axum::{routing::get, Router, Json};
use std::net::SocketAddr;
use sqlx::PgPool;
use dotenvy::dotenv;
use std::env;
use tracing_subscriber::{fmt, EnvFilter};

mod routes;
mod models;
mod repositories;
mod services;

use repositories::postgres::PostgresUserRepo;
use services::auth::AuthService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    // enforce JWT secret at startup
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set for the Identity Service");

    // create repository and auth service
    let user_repo = PostgresUserRepo::new(pool.clone());
    let user_repo_arc: Arc<dyn repositories::UserRepo> = Arc::new(user_repo);
    let auth_service = Arc::new(AuthService::new(user_repo_arc, jwt_secret, 3600));

    let app_state = AppState { auth: auth_service.clone() };

    use axum::middleware;
    // apply auth middleware to /users routes using the auth service
    let users_router = routes::user_routes()
        .layer(middleware::from_fn_with_state(app_state.auth.clone(), routes::auth_middleware));

    let app = Router::new()
        .route("/health", get(|| async { Json("ok") }))
        .nest("/auth", routes::auth_routes())
        .nest("/users", users_router)
        .with_state(app_state.clone());

    let addr = SocketAddr::from(([127,0,0,1], 4608));
    tracing::info!("Starting Identity Service at {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
