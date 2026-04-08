use axum::extract::State;
use axum::Json;

use crate::app::AppState;
use crate::common::db_err;
use crate::error::AppError;
use crate::extractors::SessionUser;
use crate::middleware::trace_id::TraceId;
use axum::Extension;
use fieldtrace_shared::*;

// GET /profile/privacy-preferences — read own preferences
pub async fn get_privacy(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
) -> Result<Json<PrivacyPreferencesResponse>, AppError> {
    let t = &tid.0;

    // Upsert a default row if none exists (lazy initialization).
    sqlx::query(
        "INSERT OR IGNORE INTO privacy_preferences (user_id) VALUES (?)"
    )
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(db_err(t))?;

    let row: (i64, i64, i64, i64, String) = sqlx::query_as(
        "SELECT show_email, show_phone, allow_audit_log_export, allow_data_sharing, updated_at \
         FROM privacy_preferences WHERE user_id = ?"
    )
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(db_err(t))?;

    Ok(Json(PrivacyPreferencesResponse {
        show_email: row.0 != 0,
        show_phone: row.1 != 0,
        allow_audit_log_export: row.2 != 0,
        allow_data_sharing: row.3 != 0,
        updated_at: row.4,
    }))
}

// PATCH /profile/privacy-preferences — update own preferences
pub async fn update_privacy(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Json(body): Json<PrivacyPreferencesUpdate>,
) -> Result<Json<PrivacyPreferencesResponse>, AppError> {
    let t = &tid.0;

    // Ensure a row exists before updating.
    sqlx::query(
        "INSERT OR IGNORE INTO privacy_preferences (user_id) VALUES (?)"
    )
    .bind(&user.user_id)
    .execute(&state.db)
    .await
    .map_err(db_err(t))?;

    // Build partial update — only supplied fields are changed.
    if let Some(v) = body.show_email {
        sqlx::query("UPDATE privacy_preferences SET show_email = ?, updated_at = datetime('now') WHERE user_id = ?")
            .bind(if v { 1 } else { 0 }).bind(&user.user_id)
            .execute(&state.db).await.map_err(db_err(t))?;
    }
    if let Some(v) = body.show_phone {
        sqlx::query("UPDATE privacy_preferences SET show_phone = ?, updated_at = datetime('now') WHERE user_id = ?")
            .bind(if v { 1 } else { 0 }).bind(&user.user_id)
            .execute(&state.db).await.map_err(db_err(t))?;
    }
    if let Some(v) = body.allow_audit_log_export {
        sqlx::query("UPDATE privacy_preferences SET allow_audit_log_export = ?, updated_at = datetime('now') WHERE user_id = ?")
            .bind(if v { 1 } else { 0 }).bind(&user.user_id)
            .execute(&state.db).await.map_err(db_err(t))?;
    }
    if let Some(v) = body.allow_data_sharing {
        sqlx::query("UPDATE privacy_preferences SET allow_data_sharing = ?, updated_at = datetime('now') WHERE user_id = ?")
            .bind(if v { 1 } else { 0 }).bind(&user.user_id)
            .execute(&state.db).await.map_err(db_err(t))?;
    }

    crate::modules::audit::write(
        &state.db, &user.user_id, "profile.privacy_updated", "user", &user.user_id, t,
    ).await;

    // Return the updated row.
    let row: (i64, i64, i64, i64, String) = sqlx::query_as(
        "SELECT show_email, show_phone, allow_audit_log_export, allow_data_sharing, updated_at \
         FROM privacy_preferences WHERE user_id = ?"
    )
    .bind(&user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(db_err(t))?;

    Ok(Json(PrivacyPreferencesResponse {
        show_email: row.0 != 0,
        show_phone: row.1 != 0,
        allow_audit_log_export: row.2 != 0,
        allow_data_sharing: row.3 != 0,
        updated_at: row.4,
    }))
}
