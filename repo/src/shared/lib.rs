use serde::{Deserialize, Serialize};

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

/// Standard error response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: u16,
    pub code: String,
    pub message: String,
    pub trace_id: String,
}

// ── Auth DTOs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub message: String,
}

// ── Users DTOs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub role: Option<String>,
}

// ── Address Book DTOs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressRequest {
    pub label: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip_plus4: String,
    pub phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// ── Intake DTOs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeRequest {
    pub intake_type: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeResponse {
    pub id: String,
    pub facility_id: String,
    pub intake_type: String,
    pub status: String,
    pub details: String,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdateRequest {
    pub status: String,
}

// ── Inspection DTOs ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionRequest {
    pub intake_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResponse {
    pub id: String,
    pub intake_id: String,
    pub inspector_id: String,
    pub status: String,
    pub outcome_notes: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveInspectionRequest {
    pub status: String,
    pub outcome_notes: String,
}

pub struct AddressResponse {
    pub id: String,
    pub label: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip_plus4: String,
    pub phone_masked: String,
    pub created_at: String,
}
