use serde::{Deserialize, Serialize};
use thiserror::Error;
use axum::Json;

pub mod browser_manager;

/// Canonical request model for launching dashboards and workspace-scoped actions.
///
/// Validation rules:
/// - dashboard_id must be a non-empty trimmed string
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchRequest {
    pub dashboard_id: String,
    pub workspace: Option<String>,
    pub browser_id: Option<String>,
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
        if let Some(b) = &self.browser_id {
            if b.is_empty() || b.len() > 64 || !b.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                return Err(LaunchError::InvalidRequest("invalid browser_id".into()));
            }
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

pub mod policy;

pub mod handlers {
    use super::{ApiResponse, LaunchRequest};
    use axum::{extract::Path, Json};
    use axum::http::StatusCode;
    use crate::browser_manager::BrowserManager;
    use crate::policy::PolicyManager;
    use axum::http::HeaderMap;

    /// Health check handler.
    pub async fn health() -> Json<ApiResponse<&'static str>> {
        Json(ApiResponse { ok: true, data: Some("ok"), error: None })
    }

    /// Handler to accept a dashboard launch request.
    /// Validates the incoming payload and returns 400 for invalid requests, 202 when accepted.
    /// If a browser_id is provided it is treated as a privileged operation and requires authorization.
    pub async fn launch_dashboard(headers: HeaderMap, Json(req): Json<LaunchRequest>) -> (StatusCode, Json<ApiResponse<String>>) {
        if let Err(e) = req.validate() {
            return (StatusCode::BAD_REQUEST, Json(ApiResponse { ok: false, data: None, error: Some(e.to_string()) }));
        }

        let policy = PolicyManager::new();
        let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());

        if req.browser_id.is_some() && !policy.is_authorized(auth_header) {
            return (StatusCode::UNAUTHORIZED, Json(ApiResponse { ok: false, data: None, error: Some("missing or invalid authorization for privileged operation".into()) }));
        }

        // If browser specified, ensure it is available
        if let Some(ref bid) = req.browser_id {
            let manager = BrowserManager::new();
            let available = manager.detect_all();
            if !available.iter().any(|b| b.id == *bid && b.installed) {
                return (StatusCode::CONFLICT, Json(ApiResponse { ok: false, data: None, error: Some("requested browser not installed".into()) }));
            }
        }

        tracing::info!(dashboard = %req.dashboard_id, workspace = ?req.workspace, browser = ?req.browser_id, "launch request received");
        // In a full implementation this would enqueue a background job. Here we accept.
        (StatusCode::ACCEPTED, Json(ApiResponse { ok: true, data: Some(format!("enqueued: {}", req.dashboard_id)), error: None }))
    }

    /// Handler to 'open' a named browser provider. Validates the name input and requires authorization.
    pub async fn open_browser(headers: HeaderMap, Path(name): Path<String>) -> (StatusCode, Json<ApiResponse<String>>) {
        // Basic validation: limit length and allowed characters to avoid injection risks.
        if name.len() == 0 || name.len() > 64 || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return (StatusCode::BAD_REQUEST, Json(ApiResponse { ok: false, data: None, error: Some("invalid browser name".into()) }));
        }

        let policy = PolicyManager::new();
        let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());
        if !policy.is_authorized(auth_header) {
            return (StatusCode::UNAUTHORIZED, Json(ApiResponse { ok: false, data: None, error: Some("missing or invalid authorization for privileged operation".into()) }));
        }

        tracing::info!(browser = %name, "open_browser called");
        (StatusCode::OK, Json(ApiResponse { ok: true, data: Some(format!("opened browser: {}", name)), error: None }))
    }

    /// Handler to list detected browsers via BrowserManager.
    pub async fn list_browsers() -> (StatusCode, Json<ApiResponse<Vec<crate::browser_manager::BrowserInfo>>>) {
        let manager = BrowserManager::new();
        let list = manager.detect_all();
        (StatusCode::OK, Json(ApiResponse { ok: true, data: Some(list), error: None }))
    }
}
