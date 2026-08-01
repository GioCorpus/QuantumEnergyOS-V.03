#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize};
use tauri::Manager;
use std::sync::Mutex;

#[derive(Deserialize, serde::Serialize)]
struct LaunchPayload {
    dashboard_id: String,
    workspace: Option<String>,
    browser_id: Option<String>,
}

// App state to hold an optional admin token for privileged operations
struct AdminToken(Mutex<Option<String>>);

#[tauri::command]
fn set_admin_token(token: Option<String>, state: tauri::State<'_, AdminToken>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| format!("lock error: {}", e))?;
    *guard = token;
    Ok(())
}

#[tauri::command]
fn launch_dashboard(payload: LaunchPayload, app_handle: tauri::AppHandle, state: tauri::State<'_, AdminToken>) -> Result<String, String> {
    // Forward the request to the backend HTTP control plane synchronously using reqwest::blocking.
    tracing::info!("Tauri launch_dashboard invoked: {}", payload.dashboard_id);
    let client = reqwest::blocking::Client::new();
    let url = "http://127.0.0.1:4607/api/launch-dashboard";
    let mut req = client.post(url).json(&payload);

    // Attach Authorization header if admin token is set
    if let Ok(guard) = state.0.lock() {
        if let Some(token) = &*guard {
            req = req.bearer_auth(token.clone());
        }
    }

    let res = req.send().map_err(|e| format!("request failed: {}", e))?;

    if !res.status().is_success() {
        let text = res.text().unwrap_or_else(|_| "<no body>".into());
        return Err(format!("backend error: {} - {}", res.status(), text));
    }

    // Optionally emit an event
    let _ = app_handle.emit_all("dashboard:launched", payload.dashboard_id.clone());

    Ok("ok".into())
}

#[tauri::command]
fn list_browsers(state: tauri::State<'_, AdminToken>) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let url = "http://127.0.0.1:4607/api/browsers";
    let mut req = client.get(url);

    // Include auth if available (some endpoints may require it later)
    if let Ok(guard) = state.0.lock() {
        if let Some(token) = &*guard {
            req = req.bearer_auth(token.clone());
        }
    }

    let res = req.send().map_err(|e| format!("request failed: {}", e))?;
    if !res.status().is_success() {
        return Err(format!("backend error: {}", res.status()));
    }
    let text = res.text().map_err(|e| format!("body read failed: {}", e))?;
    Ok(text)
}

fn main() {
    tauri::Builder::default()
        .manage(AdminToken(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![launch_dashboard, list_browsers, set_admin_token])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
