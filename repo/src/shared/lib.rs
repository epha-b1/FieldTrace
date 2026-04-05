use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse { pub status: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: u16,
    pub code: String,
    pub message: String,
    pub trace_id: String,
}

// ── Auth ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest { pub username: String, pub password: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest { pub username: String, pub password: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest { pub current_password: String, pub new_password: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse { pub id: String, pub username: String, pub role: String, pub created_at: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse { pub user: UserResponse, pub message: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest { pub username: String, pub password: String, pub role: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest { pub role: Option<String> }

// ── Address Book ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressRequest {
    pub label: String, pub street: String, pub city: String,
    pub state: String, pub zip_plus4: String, pub phone: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressResponse {
    pub id: String, pub label: String, pub street: String, pub city: String,
    pub state: String, pub zip_plus4: String, pub phone_masked: String, pub created_at: String,
}

// ── Intake ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeRequest { pub intake_type: String, pub details: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeResponse {
    pub id: String, pub facility_id: String, pub intake_type: String,
    pub status: String, pub details: String, pub created_by: String, pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdateRequest { pub status: String }

// ── Inspections ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionRequest { pub intake_id: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResponse {
    pub id: String, pub intake_id: String, pub inspector_id: String,
    pub status: String, pub outcome_notes: String, pub created_at: String, pub resolved_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveInspectionRequest { pub status: String, pub outcome_notes: String }

// ── Evidence ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadStartRequest {
    pub filename: String, pub media_type: String,
    pub total_size: i64, pub duration_seconds: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadStartResponse { pub upload_id: String, pub chunk_size_bytes: i64, pub total_chunks: i64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadChunkRequest { pub upload_id: String, pub chunk_index: i64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadCompleteRequest {
    pub upload_id: String, pub fingerprint: String, pub total_size: i64,
    pub exif_capture_time: Option<String>, pub tags: Option<String>, pub keyword: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceResponse {
    pub id: String, pub filename: String, pub media_type: String,
    pub watermark_text: String, pub missing_exif: bool,
    pub linked: bool, pub legal_hold: bool, pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceLinkRequest { pub target_type: String, pub target_id: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalHoldRequest { pub legal_hold: bool }

// ── Supply ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyRequest {
    pub name: String, pub sku: Option<String>, pub size: String, pub color: String,
    pub price_cents: Option<i64>, pub discount_cents: Option<i64>, pub notes: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyResponse {
    pub id: String, pub name: String, pub sku: Option<String>,
    pub canonical_size: Option<String>, pub canonical_color: Option<String>,
    pub price_cents: Option<i64>, pub parse_status: String, pub parse_conflicts: String, pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyResolveRequest {
    pub canonical_color: Option<String>, pub canonical_size: Option<String>,
}

// ── Traceability ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCodeRequest { pub intake_id: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCodeResponse {
    pub id: String, pub code: String, pub intake_id: Option<String>,
    pub status: String, pub version: i64, pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracePublishRequest { pub comment: String }

// ── Check-In ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRequest { pub member_id: String, pub name: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberResponse { pub id: String, pub member_id: String, pub name: String, pub created_at: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinRequest { pub member_id: String, pub override_reason: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinResponse {
    pub id: String, pub member_id: String, pub checked_in_at: String, pub was_override: bool,
}
