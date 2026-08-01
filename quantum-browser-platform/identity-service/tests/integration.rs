use testcontainers::{clients, images::postgres::Postgres};
use std::time::Duration;
use sqlx::PgPool;
use std::fs;
use std::net::TcpListener;

#[tokio::test]
async fn integration_register_login_me() -> anyhow::Result<()> {
    // start docker postgres
    let docker = clients::Cli::default();
    let node = docker.run(Postgres::default());
    let port = node.get_host_port_ipv4(5432);
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    // wait for db
    let pool = loop {
        match PgPool::connect(&database_url).await {
            Ok(p) => break p,
            Err(_) => tokio::time::sleep(Duration::from_millis(200)).await,
        }
    };

    // run migrations: read sql file and execute statements
    let sql = fs::read_to_string("./sql/migrations/001_create_users.sql")?;
    for stmt in sql.split(';') {
        let s = stmt.trim();
        if s.is_empty() { continue; }
        sqlx::query(s).execute(&pool).await?;
    }

    // start service on ephemeral port
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let addr = listener.local_addr()?;

    // create repository and auth service for the app state
    let user_repo = identity_service::repositories::postgres::PostgresUserRepo::new(pool.clone());
    let user_repo_arc: std::sync::Arc<dyn identity_service::repositories::UserRepo> = std::sync::Arc::new(user_repo);
    let auth_service = std::sync::Arc::new(identity_service::services::auth::AuthService::new(user_repo_arc, "test-secret".into(), 3600));
    let app_state = identity_service::AppState { auth: auth_service };

    let app = axum::Router::new()
        .route("/health", axum::routing::get(|| async { axum::Json("ok") }))
        .nest("/auth", identity_service::routes::auth_routes())
        .nest("/users", identity_service::routes::user_routes())
        .with_state(app_state.clone());
    let server = axum::Server::from_tcp(listener)?.serve(app.into_make_service());
    let server_handle = tokio::spawn(async move { server.await.map_err(|e| anyhow::anyhow!(e)) });

    let client = reqwest::Client::new();
    let base = format!("http://{}", addr);

    // register
    let reg = client.post(&format!("{}/auth/register", base))
        .json(&serde_json::json!({"email":"test@example.com","password":"password"}))
        .send().await?;
    assert_eq!(reg.status().as_u16(), 201);

    // login
    let login = client.post(&format!("{}/auth/login", base))
        .json(&serde_json::json!({"email":"test@example.com","password":"password"}))
        .send().await?;
    assert_eq!(login.status().as_u16(), 200);
    let body: serde_json::Value = login.json().await?;
    let token = body.get("token").and_then(|t| t.as_str()).ok_or_else(|| anyhow::anyhow!("no token"))?;

    // call /users/me with token
    let me = client.get(&format!("{}/users/me", base))
        .bearer_auth(token)
        .send().await?;
    assert_eq!(me.status().as_u16(), 200);
    let me_body: serde_json::Value = me.json().await?;
    assert!(me_body.get("user").is_some());

    // without token
    let me2 = client.get(&format!("{}/users/me", base)).send().await?;
    assert_eq!(me2.status().as_u16(), 401);

    // cleanup
    server_handle.abort();
    Ok(())
}
