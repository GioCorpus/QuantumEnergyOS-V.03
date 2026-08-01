use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UserPublic {
    pub id: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub struct NewUser {
    pub email: String,
    pub password_hash: String,
}
