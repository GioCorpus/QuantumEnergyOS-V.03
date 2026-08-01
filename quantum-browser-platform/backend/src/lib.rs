use serde::{Deserialize, Serialize};
use thiserror::Error;
use axum::Json;

/// Canonical request model for launching dashboards and workspace-scoped actions.
///
/// Validation rules:
/// - dashboard_id must be a non-empty trimmed string
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchRequest {
    pub dashboard_id: String,
    pub workspace: Option<String>,
}

/// Errors returned by LaunchRequest validation and related operations.
#[derive(Debug, Error)]
pub enum LaunchError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

impl LaunchRequest {
    /// Validate the request according to the public contract.
    ///
    /// Returns Ok(()) when the request is valid, or a typed LaunchError when invalid.
    pub fn validate(&self) -> Result<(), LaunchError> {
        if self.dashboard_id.trim().is_empty() {
            return Err(LaunchError::InvalidRequest("dashboard_id must not be empty".into()));
        }
        Ok(())
    }
}

/// Common API response envelope used by handlers in this crate.
#[derive(serde::Serialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

pub mod handlers {
    use super::{ApiResponse, LaunchRequest};
    use axum::{extract::Path, Json};
    use axum::http::StatusCode;

    /// Health check handler.
    pub async fn health() -> Json<ApiResponse<&'static str>> {
        Json(ApiResponse { ok: true, data: Some("ok"), error: None })
    }

    /// Handler to accept a dashboard launch request.
    /// Validates the incoming payload and returns 400 for invalid requests, 202 when accepted.
    pub async fn launch_dashboard(Json(req): Json<LaunchRequest>) -> (StatusCode, Json<ApiResponse<String>>) {
        if let Err(e) = req.validate() {
            return (StatusCode::BAD_REQUEST, Json(ApiResponse { ok: false, data: None, error: Some(e.to_string()) }));
        }

        tracing::info!(dashboard = %req.dashboard_id, workspace = ?req.workspace, "launch request received");
        // In a full implementation this would enqueue a background job. Here we accept.
        (StatusCode::ACCEPTED, Json(ApiResponse { ok: true, data: Some(format!("enqueued: {}", req.dashboard_id)), error: None }))
    }

    /// Handler to 'open' a named browser provider. Validates the name input.
    pub async fn open_browser(Path(name): Path<String>) -> (StatusCode, Json<ApiResponse<String>>) {
        // Basic validation: limit length and allowed characters to avoid injection risks.
        if name.len() == 0 || name.len() > 64 || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return (StatusCode::BAD_REQUEST, Json(ApiResponse { ok: false, data: None, error: Some("invalid browser name".into()) }));
        }
        tracing::info!(browser = %name, "open_browser called");
        (StatusCode::OK, Json(ApiResponse { ok: true, data: Some(format!("opened browser: {}", name)), error: None }))
    }
}
