use qb_platform_backend::handlers::{launch_dashboard, health};
use qb_platform_backend::LaunchRequest;
use axum::Router;
use axum::routing::get;
use axum::routing::post;
use tower::util::ServiceExt; // for `oneshot`
use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;

#[tokio::test]
async fn launch_dashboard_http_enqueued() {
    let app = Router::<()>::new()
        .route("/health", get(health))
        .route("/api/launch-dashboard", post(launch_dashboard));

    let payload = LaunchRequest { dashboard_id: "quantum-dashboard".into(), workspace: Some("research".into()), browser_id: None };
    let body = serde_json::to_string(&payload).expect("serialize payload");

    let req = Request::builder()
        .method("POST")
        .uri("/api/launch-dashboard")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("request builder");

    let resp = app.oneshot(req).await.expect("router oneshot");
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
}
