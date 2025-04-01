use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JSON payload for authentication request.
#[derive(Serialize, Deserialize)]
pub(crate) struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// JSON payload for authentication request.
#[derive(Serialize, Deserialize)]
pub(crate) struct LogoutRequest {
    pub session_key: Uuid,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SignupRequest {
    pub email: String,
    pub password: String,
}

/// JSON response containing the generated private key.
#[derive(Serialize)]
pub(crate) struct AuthResponse {
    pub session_key: Uuid,
}
