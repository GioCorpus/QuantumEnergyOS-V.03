use std::process::Command;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BrowserInfo {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub path: Option<String>,
}

#[async_trait::async_trait]
pub trait BrowserProvider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn detect(&self) -> BrowserInfo;
}

pub struct BraveProvider;
impl BraveProvider {
    pub fn new() -> Self { Self }
}

impl BrowserProvider for BraveProvider {
    fn id(&self) -> &str { "brave" }
    fn name(&self) -> &str { "Brave" }

    fn detect(&self) -> BrowserInfo {
        // On Unix-like systems Brave is usually 'brave' in PATH. On Windows it may be in Program Files.
        // This is a best-effort detection for the scaffold.
        #[cfg(target_family = "windows")]
        let maybe = {
            let program_files = std::env::var("PROGRAMFILES").unwrap_or_default();
            let p = format!("{}\\BraveSoftware\\Brave-Browser\\Application\\brave.exe", program_files);
            if std::path::Path::new(&p).exists() { Some(p) } else { None }
        };
        #[cfg(not(target_family = "windows"))]
        let maybe = which::which("brave").ok().map(|p| p.to_string_lossy().to_string());

        BrowserInfo { id: self.id().to_string(), name: self.name().to_string(), installed: maybe.is_some(), path: maybe }
    }
}

pub struct LibreWolfProvider;
impl LibreWolfProvider {
    pub fn new() -> Self { Self }
}

impl BrowserProvider for LibreWolfProvider {
    fn id(&self) -> &str { "librewolf" }
    fn name(&self) -> &str { "LibreWolf" }

    fn detect(&self) -> BrowserInfo {
        #[cfg(target_family = "windows")]
        let maybe = {
            let program_files = std::env::var("PROGRAMFILES").unwrap_or_default();
            let p = format!("{}\\LibreWolf\\librewolf.exe", program_files);
            if std::path::Path::new(&p).exists() { Some(p) } else { None }
        };
        #[cfg(not(target_family = "windows"))]
        let maybe = which::which("librewolf").ok().map(|p| p.to_string_lossy().to_string());

        BrowserInfo { id: self.id().to_string(), name: self.name().to_string(), installed: maybe.is_some(), path: maybe }
    }
}

pub struct BrowserManager {
    providers: Vec<Box<dyn BrowserProvider>>,
}

impl BrowserManager {
    pub fn new() -> Self {
        Self { providers: vec![Box::new(BraveProvider::new()), Box::new(LibreWolfProvider::new())] }
    }

    pub fn detect_all(&self) -> Vec<BrowserInfo> {
        self.providers.iter().map(|p| p.detect()).collect()
    }
}
