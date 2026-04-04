use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::AppError;
use crate::extractors::SessionUser;
use crate::middleware::trace_id::TraceId;
use fieldtrace_shared::*;

pub async fn list(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
) -> Result<Json<Vec<InspectionResponse>>, AppError> {
    let rows = sqlx::query_as::<_, InspRow>(
        "SELECT id, intake_id, inspector_id, status, outcome_notes, created_at, resolved_at FROM inspections ORDER BY created_at DESC",
    ).fetch_all(&state.db).await
    .map_err(|e| AppError::internal(e.to_string(), &tid.0))?;
    Ok(Json(rows.into_iter().map(to_resp).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Json(body): Json<InspectionRequest>,
) -> Result<(StatusCode, Json<InspectionResponse>), AppError> {
    let t = &tid.0;
    // Verify intake exists
    let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM intake_records WHERE id = ?")
        .bind(&body.intake_id).fetch_optional(&state.db).await
        .map_err(|e| AppError::internal(e.to_string(), t))?;
    if exists.is_none() {
        return Err(AppError::not_found("Intake record not found", t));
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO inspections (id, intake_id, inspector_id) VALUES (?,?,?)")
        .bind(&id).bind(&body.intake_id).bind(&user.user_id)
        .execute(&state.db).await.map_err(|e| AppError::internal(e.to_string(), t))?;

    Ok((StatusCode::CREATED, Json(InspectionResponse {
        id, intake_id: body.intake_id, inspector_id: user.user_id,
        status: "pending".into(), outcome_notes: String::new(),
        created_at: String::new(), resolved_at: None,
    })))
}

pub async fn resolve(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Path(id): Path<String>,
    Json(body): Json<ResolveInspectionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let t = &tid.0;
    if !["passed", "failed"].contains(&body.status.as_str()) {
        return Err(AppError::validation("Status must be passed or failed", t));
    }

    let current: Option<(String,)> = sqlx::query_as("SELECT status FROM inspections WHERE id = ?")
        .bind(&id).fetch_optional(&state.db).await
        .map_err(|e| AppError::internal(e.to_string(), t))?;

    match current {
        None => return Err(AppError::not_found("Inspection not found", t)),
        Some((s,)) if s != "pending" => return Err(AppError::conflict(
            format!("Inspection already resolved as '{}'", s), t,
        )),
        _ => {}
    }

    sqlx::query("UPDATE inspections SET status = ?, outcome_notes = ?, resolved_at = datetime('now') WHERE id = ?")
        .bind(&body.status).bind(&body.outcome_notes).bind(&id)
        .execute(&state.db).await.map_err(|e| AppError::internal(e.to_string(), t))?;

    Ok(Json(serde_json::json!({"message": "Inspection resolved"})))
}

fn to_resp(r: InspRow) -> InspectionResponse {
    InspectionResponse {
        id: r.id, intake_id: r.intake_id, inspector_id: r.inspector_id,
        status: r.status, outcome_notes: r.outcome_notes,
        created_at: r.created_at, resolved_at: r.resolved_at,
    }
}

#[derive(sqlx::FromRow)]
struct InspRow {
    id: String, intake_id: String, inspector_id: String,
    status: String, outcome_notes: String, created_at: String,
    resolved_at: Option<String>,
}
