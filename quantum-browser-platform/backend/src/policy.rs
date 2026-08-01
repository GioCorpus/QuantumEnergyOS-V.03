pub struct PolicyManager {
    admin_token: String,
}

impl PolicyManager {
    pub fn new() -> Self {
        let t = std::env::var("QBP_ADMIN_TOKEN").unwrap_or_else(|_| "secret".into());
        Self { admin_token: t }
    }

    /// Check if the provided Authorization header value corresponds to the admin token.
    /// The header should be of the form "Bearer <token>".
    pub fn is_authorized(&self, header: Option<&str>) -> bool {
        if let Some(h) = header {
            let expected = format!("Bearer {}", self.admin_token);
            return h == expected;
        }
        false
    }

    /// Policy check placeholder. Returns true if the operation is permitted in the given workspace.
    /// Extend this with real policy rules backed by files or a database.
    pub fn can_perform(&self, _operation: &str, _workspace: Option<&str>) -> bool {
        // For the scaffold, allow by default. Administrator-only operations should still require auth via is_authorized.
        true
    }
}
