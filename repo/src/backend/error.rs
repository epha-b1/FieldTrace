use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::Map;

#[derive(Debug, Serialize, Clone)]
pub struct ErrorBody {
    pub status: u16,
    pub code: String,
    pub message: String,
    pub trace_id: String,
    /// Extra structured fields merged into the response (flattened) for
    /// error-code-specific payloads like `retry_after_seconds` on
    /// anti-passback. Always a (possibly empty) map — safe to flatten.
    #[serde(flatten)]
    pub extra: Map<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub trace_id: String,
    pub extra: Map<String, serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            status: self.status.as_u16(),
            code: self.code.to_string(),
            message: self.message,
            trace_id: self.trace_id,
            extra: self.extra,
        };
        (self.status, Json(body)).into_response()
    }
}

impl AppError {
    pub fn validation(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_REQUEST, code: "VALIDATION_ERROR", message: msg.into(), trace_id: tid.into(), extra: Map::new() }
    }
    pub fn unauthorized(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::UNAUTHORIZED, code: "UNAUTHORIZED", message: msg.into(), trace_id: tid.into(), extra: Map::new() }
    }
    pub fn forbidden(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::FORBIDDEN, code: "FORBIDDEN", message: msg.into(), trace_id: tid.into(), extra: Map::new() }
    }
    pub fn not_found(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::NOT_FOUND, code: "NOT_FOUND", message: msg.into(), trace_id: tid.into(), extra: Map::new() }
    }
    pub fn conflict(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::CONFLICT, code: "CONFLICT", message: msg.into(), trace_id: tid.into(), extra: Map::new() }
    }
    pub fn locked(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::TOO_MANY_REQUESTS, code: "ACCOUNT_LOCKED", message: msg.into(), trace_id: tid.into(), extra: Map::new() }
    }
    pub fn internal(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, code: "INTERNAL_ERROR", message: msg.into(), trace_id: tid.into(), extra: Map::new() }
    }
    /// Chainable helper to attach structured extra data. Accepts any
    /// JSON Object and merges its keys into the error envelope.
    pub fn with_extra(mut self, value: serde_json::Value) -> Self {
        if let serde_json::Value::Object(map) = value {
            for (k, v) in map { self.extra.insert(k, v); }
        }
        self
    }
    /// Custom status + code escape hatch for domain-specific codes
    /// (e.g. ANTI_PASSBACK, NEEDS_REVIEW).
    pub fn custom(status: StatusCode, code: &'static str, msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status, code, message: msg.into(), trace_id: tid.into(), extra: Map::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_body_flattens_extra_fields() {
        let body = ErrorBody {
            status: 409,
            code: "ANTI_PASSBACK".into(),
            message: "Retry later".into(),
            trace_id: "abc".into(),
            extra: {
                let mut m = Map::new();
                m.insert("retry_after_seconds".into(), serde_json::json!(60));
                m
            },
        };
        let s = serde_json::to_string(&body).unwrap();
        assert!(s.contains("\"status\":409"));
        assert!(s.contains("\"code\":\"ANTI_PASSBACK\""));
        assert!(s.contains("\"trace_id\":\"abc\""));
        assert!(s.contains("\"retry_after_seconds\":60"));
        // No nested "extra" key
        assert!(!s.contains("\"extra\""));
    }

    #[test]
    fn empty_extra_produces_no_nested_key() {
        let body = ErrorBody {
            status: 400,
            code: "VALIDATION_ERROR".into(),
            message: "Bad input".into(),
            trace_id: "xyz".into(),
            extra: Map::new(),
        };
        let s = serde_json::to_string(&body).unwrap();
        assert!(!s.contains("\"extra\""));
        assert!(s.contains("\"status\":400"));
    }

    #[test]
    fn with_extra_merges_object_fields() {
        let e = AppError::custom(
            StatusCode::CONFLICT, "ANTI_PASSBACK", "blocked", "tid1",
        ).with_extra(serde_json::json!({"retry_after_seconds": 30}));
        assert_eq!(e.extra.get("retry_after_seconds").and_then(|v| v.as_i64()), Some(30));
    }
}
