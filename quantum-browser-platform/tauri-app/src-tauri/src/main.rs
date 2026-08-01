#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize};
use tauri::Manager;

#[derive(Deserialize, serde::Serialize)]
struct LaunchPayload {
    dashboard_id: String,
    workspace: Option<String>,
    browser_id: Option<String>,
}

#[tauri::command]
fn launch_dashboard(payload: LaunchPayload, app_handle: tauri::AppHandle) -> Result<String, String> {
    // Forward the request to the backend HTTP control plane synchronously using reqwest::blocking.
    tracing::info!("Tauri launch_dashboard invoked: {}", payload.dashboard_id);
    let client = reqwest::blocking::Client::new();
    let url = "http://127.0.0.1:4607/api/launch-dashboard";
    let res = client.post(url)
        .json(&payload)
        .send()
        .map_err(|e| format!("request failed: {}", e))?;

    if !res.status().is_success() {
        let text = res.text().unwrap_or_else(|_| "<no body>".into());
        return Err(format!("backend error: {} - {}", res.status(), text));
    }

    // Optionally emit an event
    let _ = app_handle.emit_all("dashboard:launched", payload.dashboard_id.clone());

    Ok("ok".into())
}

#[tauri::command]
fn list_browsers() -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let url = "http://127.0.0.1:4607/api/browsers";
    let res = client.get(url).send().map_err(|e| format!("request failed: {}", e))?;
    if !res.status().is_success() {
        return Err(format!("backend error: {}", res.status()));
    }
    let text = res.text().map_err(|e| format!("body read failed: {}", e))?;
    Ok(text)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![launch_dashboard])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
