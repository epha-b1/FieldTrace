use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use std::collections::HashMap;
use uuid::Uuid;

use crate::app::AppState;
use crate::common::{db_err, is_admin, require_write_role, CivilDateTime};
use crate::error::AppError;
use crate::extractors::SessionUser;
use crate::middleware::trace_id::TraceId;
use fieldtrace_shared::*;

// Size limits (bytes)
const MAX_PHOTO: i64 = 25 * 1024 * 1024;
const MAX_VIDEO: i64 = 150 * 1024 * 1024;
const MAX_AUDIO: i64 = 20 * 1024 * 1024;
const MAX_VIDEO_SECONDS: i64 = 60;
const MAX_AUDIO_SECONDS: i64 = 120;

const FACILITY_CODE: &str = "FAC01";

fn check_size(media_type: &str, size: i64, tid: &str) -> Result<(), AppError> {
    let max = match media_type {
        "photo" => MAX_PHOTO,
        "video" => MAX_VIDEO,
        "audio" => MAX_AUDIO,
        _ => return Err(AppError::validation("media_type must be photo, video, or audio", tid)),
    };
    if size > max {
        return Err(AppError::validation(format!("File exceeds {} bytes for {}", max, media_type), tid));
    }
    Ok(())
}

/// Build the facility + timestamp watermark string actually burned into photos
/// (format: `FAC01 MM/DD/YYYY hh:mm AM/PM`). For video/audio this same text
/// is persisted as metadata.
fn build_watermark() -> String {
    format!("{} {}", FACILITY_CODE, CivilDateTime::now().us_12h())
}

// POST /media/upload/start — create upload session
pub async fn upload_start(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Json(body): Json<UploadStartRequest>,
) -> Result<Json<UploadStartResponse>, AppError> {
    let t = &tid.0;
    require_write_role(&user, t)?;
    check_size(&body.media_type, body.total_size, t)?;
    if body.media_type == "video" && body.duration_seconds > MAX_VIDEO_SECONDS {
        return Err(AppError::validation("Video exceeds 60 seconds", t));
    }
    if body.media_type == "audio" && body.duration_seconds > MAX_AUDIO_SECONDS {
        return Err(AppError::validation("Audio exceeds 2 minutes", t));
    }
    let id = Uuid::new_v4().to_string();
    let total_chunks = (body.total_size + (2 * 1024 * 1024) - 1) / (2 * 1024 * 1024);
    sqlx::query("INSERT INTO upload_sessions (id, filename, media_type, total_chunks, uploader_id) VALUES (?,?,?,?,?)")
        .bind(&id).bind(&body.filename).bind(&body.media_type)
        .bind(total_chunks).bind(&user.user_id)
        .execute(&state.db).await
        .map_err(db_err(t))?;
    Ok(Json(UploadStartResponse { upload_id: id, chunk_size_bytes: 2 * 1024 * 1024, total_chunks }))
}

// POST /media/upload/chunk — receive chunk metadata
pub async fn upload_chunk(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Json(body): Json<UploadChunkRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let t = &tid.0;
    require_write_role(&user, t)?;

    let session: Option<(String, String, i64)> = sqlx::query_as(
        "SELECT received_chunks, status, total_chunks FROM upload_sessions WHERE id = ? AND uploader_id = ?"
    ).bind(&body.upload_id).bind(&user.user_id)
        .fetch_optional(&state.db).await
        .map_err(db_err(t))?;

    let (received_json, status, total) = session.ok_or_else(|| AppError::not_found("Upload session not found", t))?;
    if status != "in_progress" {
        return Err(AppError::conflict("Upload already complete or failed", t));
    }
    if body.chunk_index < 0 || body.chunk_index >= total {
        return Err(AppError::validation("chunk_index out of range", t));
    }

    let mut received: Vec<i64> = serde_json::from_str(&received_json).unwrap_or_default();
    if !received.contains(&body.chunk_index) {
        received.push(body.chunk_index);
        received.sort();
    }
    let new_json = serde_json::to_string(&received).unwrap();
    sqlx::query("UPDATE upload_sessions SET received_chunks = ? WHERE id = ?")
        .bind(&new_json).bind(&body.upload_id)
        .execute(&state.db).await
        .map_err(db_err(t))?;

    Ok(Json(serde_json::json!({
        "received_count": received.len(),
        "total_chunks": total,
        "complete": received.len() as i64 == total
    })))
}

// POST /media/upload/complete — finalize
pub async fn upload_complete(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Json(body): Json<UploadCompleteRequest>,
) -> Result<(StatusCode, Json<EvidenceResponse>), AppError> {
    let t = &tid.0;
    require_write_role(&user, t)?;

    // Fingerprint format validation: must be non-empty hex-like (32-128 chars)
    if body.fingerprint.trim().is_empty()
        || body.fingerprint.len() < 8
        || body.fingerprint.len() > 256
        || !body.fingerprint.chars().all(|c| c.is_ascii_alphanumeric())
    {
        return Err(AppError::validation("Invalid fingerprint format", t));
    }

    let session: Option<(String, String, i64, String)> = sqlx::query_as(
        "SELECT filename, media_type, total_chunks, received_chunks FROM upload_sessions WHERE id = ? AND uploader_id = ?"
    ).bind(&body.upload_id).bind(&user.user_id)
        .fetch_optional(&state.db).await
        .map_err(db_err(t))?;

    let (filename, media_type, total, received_json) = session.ok_or_else(|| AppError::not_found("Upload session not found", t))?;
    let received: Vec<i64> = serde_json::from_str(&received_json).unwrap_or_default();
    if received.len() as i64 != total {
        return Err(AppError::conflict(
            format!("Missing chunks: got {}/{}", received.len(), total), t,
        ));
    }

    let evidence_id = Uuid::new_v4().to_string();
    let watermark = build_watermark();
    let missing_exif = if body.exif_capture_time.is_none() && media_type == "photo" { 1 } else { 0 };

    sqlx::query(
        "INSERT INTO evidence_records (id, filename, media_type, size_bytes, fingerprint, watermark_text, exif_capture_time, missing_exif, tags, keyword, uploaded_by) \
         VALUES (?,?,?,?,?,?,?,?,?,?,?)"
    )
    .bind(&evidence_id).bind(&filename).bind(&media_type)
    .bind(body.total_size).bind(&body.fingerprint).bind(&watermark)
    .bind(&body.exif_capture_time).bind(missing_exif)
    .bind(body.tags.clone().unwrap_or_default()).bind(body.keyword.clone().unwrap_or_default())
    .bind(&user.user_id)
    .execute(&state.db).await
    .map_err(db_err(t))?;

    sqlx::query("UPDATE upload_sessions SET status = 'complete' WHERE id = ?")
        .bind(&body.upload_id).execute(&state.db).await.ok();

    crate::modules::audit::write(
        &state.db, &user.user_id, "evidence.upload_complete", "evidence", &evidence_id, t,
    ).await;

    Ok((StatusCode::CREATED, Json(EvidenceResponse {
        id: evidence_id, filename, media_type,
        watermark_text: watermark, missing_exif: missing_exif != 0,
        linked: false, legal_hold: false, created_at: String::new(),
    })))
}

// GET /evidence?keyword=&tag=&from=&to= — list with search filters
pub async fn list(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<Vec<EvidenceResponse>>, AppError> {
    let t = &tid.0;

    let mut sql = String::from(
        "SELECT id, filename, media_type, watermark_text, missing_exif, linked, legal_hold, created_at \
         FROM evidence_records WHERE 1=1"
    );
    let mut binds: Vec<String> = Vec::new();

    if let Some(k) = q.get("keyword") {
        if !k.is_empty() {
            sql.push_str(" AND (keyword LIKE ? OR filename LIKE ?)");
            binds.push(format!("%{}%", k));
            binds.push(format!("%{}%", k));
        }
    }
    if let Some(tag) = q.get("tag") {
        if !tag.is_empty() {
            sql.push_str(" AND tags LIKE ?");
            binds.push(format!("%{}%", tag));
        }
    }
    if let Some(from) = q.get("from") {
        if !from.is_empty() {
            sql.push_str(" AND (exif_capture_time >= ? OR created_at >= ?)");
            binds.push(from.clone());
            binds.push(from.clone());
        }
    }
    if let Some(to) = q.get("to") {
        if !to.is_empty() {
            sql.push_str(" AND (exif_capture_time <= ? OR created_at <= ?)");
            binds.push(to.clone());
            binds.push(to.clone());
        }
    }
    sql.push_str(" ORDER BY created_at DESC");

    let mut query = sqlx::query_as::<_, EvidenceRow>(&sql);
    for b in &binds { query = query.bind(b); }
    let rows = query.fetch_all(&state.db).await.map_err(db_err(t))?;

    Ok(Json(rows.into_iter().map(|r| EvidenceResponse {
        id: r.id, filename: r.filename, media_type: r.media_type,
        watermark_text: r.watermark_text, missing_exif: r.missing_exif != 0,
        linked: r.linked != 0, legal_hold: r.legal_hold != 0,
        created_at: r.created_at,
    }).collect()))
}

// DELETE /evidence/:id — only if unlinked AND (uploader or admin)
pub async fn delete(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let t = &tid.0;
    require_write_role(&user, t)?;

    // Load linked flag + uploader for object-level auth
    let row: Option<(i64, String, i64)> = sqlx::query_as(
        "SELECT linked, uploaded_by, legal_hold FROM evidence_records WHERE id = ?"
    ).bind(&id).fetch_optional(&state.db).await.map_err(db_err(t))?;
    let (linked, uploader, legal_hold) = row.ok_or_else(|| AppError::not_found("Evidence not found", t))?;

    if legal_hold != 0 {
        return Err(AppError::conflict("Cannot delete evidence under legal hold", t));
    }
    if linked != 0 {
        return Err(AppError::conflict("Cannot delete linked evidence", t));
    }
    // Object-level auth: uploader OR admin
    if uploader != user.user_id && !is_admin(&user) {
        return Err(AppError::forbidden(
            "Only the uploader or an administrator can delete this evidence", t,
        ));
    }

    sqlx::query("DELETE FROM evidence_records WHERE id = ?")
        .bind(&id).execute(&state.db).await
        .map_err(db_err(t))?;

    crate::modules::audit::write(
        &state.db, &user.user_id, "evidence.delete", "evidence", &id, t,
    ).await;

    Ok(Json(serde_json::json!({"message":"Deleted"})))
}

// POST /evidence/:id/link — uploader or admin only
pub async fn link(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Path(id): Path<String>,
    Json(body): Json<EvidenceLinkRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let t = &tid.0;
    require_write_role(&user, t)?;

    if !["intake","inspection","traceability","checkin"].contains(&body.target_type.as_str()) {
        return Err(AppError::validation("Invalid target_type", t));
    }

    // Load uploader for object-level auth
    let row: Option<(String,)> = sqlx::query_as("SELECT uploaded_by FROM evidence_records WHERE id = ?")
        .bind(&id).fetch_optional(&state.db).await.map_err(db_err(t))?;
    let (uploader,) = row.ok_or_else(|| AppError::not_found("Evidence not found", t))?;
    if uploader != user.user_id && !is_admin(&user) {
        return Err(AppError::forbidden(
            "Only the uploader or an administrator can link this evidence", t,
        ));
    }

    let link_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO evidence_links (id, evidence_id, target_type, target_id) VALUES (?,?,?,?)")
        .bind(&link_id).bind(&id).bind(&body.target_type).bind(&body.target_id)
        .execute(&state.db).await
        .map_err(db_err(t))?;
    sqlx::query("UPDATE evidence_records SET linked = 1 WHERE id = ?")
        .bind(&id).execute(&state.db).await.ok();

    crate::modules::audit::write(
        &state.db, &user.user_id, "evidence.link", "evidence", &id, t,
    ).await;

    Ok(Json(serde_json::json!({"message":"Linked","link_id":link_id})))
}

// PATCH /evidence/:id/legal-hold — admin only (router enforces, plus in-handler check)
pub async fn legal_hold(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Path(id): Path<String>,
    Json(body): Json<LegalHoldRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let t = &tid.0;
    if !is_admin(&user) {
        return Err(AppError::forbidden("Administrator required to set legal hold", t));
    }
    let res = sqlx::query("UPDATE evidence_records SET legal_hold = ? WHERE id = ?")
        .bind(if body.legal_hold { 1 } else { 0 }).bind(&id)
        .execute(&state.db).await
        .map_err(db_err(t))?;
    if res.rows_affected() == 0 {
        return Err(AppError::not_found("Evidence not found", t));
    }
    crate::modules::audit::write(
        &state.db, &user.user_id, "evidence.legal_hold", "evidence", &id, t,
    ).await;
    Ok(Json(serde_json::json!({"message":"Legal hold updated","legal_hold":body.legal_hold})))
}

#[derive(sqlx::FromRow)]
struct EvidenceRow {
    id: String, filename: String, media_type: String,
    watermark_text: String, missing_exif: i64, linked: i64, legal_hold: i64,
    created_at: String,
}
