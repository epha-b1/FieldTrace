use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ErrorBody {
    pub status: u16,
    pub code: String,
    pub message: String,
    pub trace_id: String,
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub trace_id: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            status: self.status.as_u16(),
            code: self.code.to_string(),
            message: self.message,
            trace_id: self.trace_id,
        };
        (self.status, Json(body)).into_response()
    }
}

impl AppError {
    pub fn validation(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_REQUEST, code: "VALIDATION_ERROR", message: msg.into(), trace_id: tid.into() }
    }
    pub fn unauthorized(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::UNAUTHORIZED, code: "UNAUTHORIZED", message: msg.into(), trace_id: tid.into() }
    }
    pub fn forbidden(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::FORBIDDEN, code: "FORBIDDEN", message: msg.into(), trace_id: tid.into() }
    }
    pub fn not_found(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::NOT_FOUND, code: "NOT_FOUND", message: msg.into(), trace_id: tid.into() }
    }
    pub fn conflict(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::CONFLICT, code: "CONFLICT", message: msg.into(), trace_id: tid.into() }
    }
    pub fn locked(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::TOO_MANY_REQUESTS, code: "ACCOUNT_LOCKED", message: msg.into(), trace_id: tid.into() }
    }
    pub fn internal(msg: impl Into<String>, tid: impl Into<String>) -> Self {
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, code: "INTERNAL_ERROR", message: msg.into(), trace_id: tid.into() }
    }
}
