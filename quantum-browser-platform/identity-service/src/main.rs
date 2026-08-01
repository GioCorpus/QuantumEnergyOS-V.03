use axum::{routing::get, Router, Json};
use std::net::SocketAddr;
use sqlx::PgPool;
use dotenvy::dotenv;
use std::env;
use tracing_subscriber::{fmt, EnvFilter};

mod routes;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    let app = Router::new()
        .route("/health", get(|| async { Json("ok") }))
        .nest("/auth", routes::auth_routes())
        .nest("/users", routes::user_routes())
        .with_state(pool.clone());

    let addr = SocketAddr::from(([127,0,0,1], 4608));
    tracing::info!("Starting Identity Service at {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
