use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::{Extension, Json};

use crate::app::AppState;
use crate::error::AppError;
use crate::extractors::SessionUser;
use crate::middleware::trace_id::TraceId;

pub async fn list(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let t = &tid.0;
    if user.role != "administrator" && user.role != "auditor" {
        return Err(AppError::forbidden("Only Admin or Auditor", t));
    }
    let rows: Vec<(i64, Option<String>, String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT id, actor_id, action, resource_type, resource_id, created_at FROM audit_logs ORDER BY id DESC LIMIT 200"
    ).fetch_all(&state.db).await
    .map_err(crate::common::db_err(t))?;
    Ok(Json(rows.into_iter().map(|(id, actor, action, rt, rid, at)| {
        serde_json::json!({
            "id": id,
            "actor_id": actor.unwrap_or_else(|| "[REDACTED]".into()),
            "action": action,
            "resource_type": rt,
            "resource_id": rid.unwrap_or_default(),
            "created_at": at
        })
    }).collect()))
}

pub async fn export_csv(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
) -> Result<impl IntoResponse, AppError> {
    let t = &tid.0;
    if user.role != "administrator" && user.role != "auditor" {
        return Err(AppError::forbidden("Only Admin or Auditor", t));
    }
    let rows: Vec<(i64, String, String, String)> = sqlx::query_as(
        "SELECT id, action, resource_type, created_at FROM audit_logs ORDER BY id"
    ).fetch_all(&state.db).await
    .map_err(crate::common::db_err(t))?;

    // Header notes that sensitive fields are [REDACTED] in all rows
    let mut csv = String::from("# Sensitive fields (actor_id, ip, payloads) are [REDACTED]\nid,action,resource_type,created_at,sensitive_data\n");
    for (id, a, rt, at) in rows {
        csv.push_str(&format!("{},{},{},{},[REDACTED]\n", id, a, rt, at));
    }

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"));
    headers.insert("Content-Disposition", HeaderValue::from_static("attachment; filename=\"audit.csv\""));
    Ok((StatusCode::OK, headers, csv))
}
