#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize};
use tauri::Manager;

#[derive(Deserialize)]
struct LaunchPayload {
    dashboard_id: String,
    workspace: Option<String>,
}

#[tauri::command]
fn launch_dashboard(payload: LaunchPayload, app_handle: tauri::AppHandle) -> Result<String, String> {
    // In production the Tauri command would forward to the backend daemon or Browser Manager via IPC/HTTP
    tracing::info!("Tauri launch_dashboard invoked: {}", payload.dashboard_id);
    // Example: emit an event to the webview
    let _ = app_handle.emit_all("dashboard:launched", payload.dashboard_id.clone());
    Ok(format!("tauri: launched {}", payload.dashboard_id))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![launch_dashboard])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
