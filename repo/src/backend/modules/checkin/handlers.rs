use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use uuid::Uuid;

use crate::app::AppState;
use crate::common::{db_err, is_admin, require_write_role};
use crate::error::AppError;
use crate::extractors::SessionUser;
use crate::middleware::trace_id::TraceId;
use fieldtrace_shared::*;

const ANTI_PASSBACK_SECONDS: i64 = 120;

pub async fn list_members(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
) -> Result<Json<Vec<MemberResponse>>, AppError> {
    let t = &tid.0;
    let rows = sqlx::query_as::<_, MemberRow>("SELECT id, member_id, name, created_at FROM members")
        .fetch_all(&state.db).await
        .map_err(db_err(t))?;
    Ok(Json(rows.into_iter().map(|r| MemberResponse {
        id: r.id, member_id: r.member_id, name: r.name, created_at: r.created_at,
    }).collect()))
}

pub async fn create_member(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Json(body): Json<MemberRequest>,
) -> Result<(StatusCode, Json<MemberResponse>), AppError> {
    let t = &tid.0;
    require_write_role(&user, t)?;

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO members (id, member_id, name) VALUES (?,?,?)")
        .bind(&id).bind(&body.member_id).bind(&body.name)
        .execute(&state.db).await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                AppError::conflict("Member ID already exists", t)
            } else {
                tracing::error!(trace_id = %t, error = %msg, "member insert failed");
                AppError::internal("Internal server error", t)
            }
        })?;

    crate::modules::audit::write(
        &state.db, &user.user_id, "member.create", "member", &id, t,
    ).await;

    Ok((StatusCode::CREATED, Json(MemberResponse {
        id, member_id: body.member_id, name: body.name, created_at: String::new(),
    })))
}

pub async fn checkin(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Json(body): Json<CheckinRequest>,
) -> Result<(StatusCode, Json<CheckinResponse>), AppError> {
    let t = &tid.0;
    require_write_role(&user, t)?;

    // Find member
    let member: Option<(String,)> = sqlx::query_as("SELECT id FROM members WHERE member_id = ?")
        .bind(&body.member_id).fetch_optional(&state.db).await
        .map_err(db_err(t))?;
    let (member_uuid,) = member.ok_or_else(|| AppError::not_found("Member not found", t))?;

    // Validate override_reason if provided — must be non-empty.
    if let Some(ref reason) = body.override_reason {
        if reason.trim().is_empty() {
            return Err(AppError::validation(
                "override_reason must be non-empty when provided", t,
            ));
        }
    }

    // Anti-passback check (unless override)
    if body.override_reason.is_none() {
        // Fetch the most recent checkin within the anti-passback window and
        // compute the remaining wait.
        let recent: Option<(String, i64)> = sqlx::query_as(
            "SELECT checked_in_at, CAST((strftime('%s','now') - strftime('%s', checked_in_at)) AS INTEGER) \
             FROM checkin_ledger \
             WHERE member_id = ? AND facility_id = 'default' \
               AND checked_in_at > datetime('now', '-120 seconds') \
             ORDER BY checked_in_at DESC LIMIT 1"
        ).bind(&member_uuid).fetch_optional(&state.db).await
            .map_err(db_err(t))?;

        if let Some((_last_at, elapsed)) = recent {
            let retry_after = (ANTI_PASSBACK_SECONDS - elapsed).max(1);
            return Err(AppError::custom(
                StatusCode::CONFLICT,
                "ANTI_PASSBACK",
                format!("Re-entry blocked. Retry in {} seconds.", retry_after),
                t,
            ).with_extra(serde_json::json!({ "retry_after_seconds": retry_after })));
        }
    } else {
        // Only admin can override
        if !is_admin(&user) {
            return Err(AppError::forbidden("Only Administrator can override anti-passback", t));
        }
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO checkin_ledger (id, member_id, override_reason, override_by) VALUES (?,?,?,?)")
        .bind(&id).bind(&member_uuid).bind(&body.override_reason).bind(&user.user_id)
        .execute(&state.db).await
        .map_err(db_err(t))?;

    crate::modules::audit::write(
        &state.db, &user.user_id,
        if body.override_reason.is_some() { "checkin.override" } else { "checkin.create" },
        "checkin", &id, t,
    ).await;

    Ok((StatusCode::CREATED, Json(CheckinResponse {
        id, member_id: body.member_id, checked_in_at: String::new(),
        was_override: body.override_reason.is_some(),
    })))
}

pub async fn history(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
) -> Result<Json<serde_json::Value>, AppError> {
    let t = &tid.0;
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT c.id, m.member_id, c.checked_in_at FROM checkin_ledger c JOIN members m ON c.member_id = m.id ORDER BY c.checked_in_at DESC LIMIT 100"
    ).fetch_all(&state.db).await
    .map_err(db_err(t))?;
    let list: Vec<serde_json::Value> = rows.into_iter().map(|(id, mid, at)|
        serde_json::json!({"id":id,"member_id":mid,"checked_in_at":at})).collect();
    Ok(Json(serde_json::json!({"history": list})))
}

#[derive(sqlx::FromRow)]
struct MemberRow {
    id: String, member_id: String, name: String, created_at: String,
}
