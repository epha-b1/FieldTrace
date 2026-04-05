use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use std::collections::HashMap;

use crate::app::AppState;
use crate::common::{db_err, require_admin_or_auditor};
use crate::error::AppError;
use crate::extractors::SessionUser;
use crate::middleware::trace_id::TraceId;

pub async fn summary(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let t = &tid.0;

    // Optional filters: from, to (ISO date strings), status, intake_type, tag
    let from = q.get("from").cloned().unwrap_or_default();
    let to = q.get("to").cloned().unwrap_or_default();
    let status = q.get("status").cloned().unwrap_or_default();
    let typ = q.get("intake_type").cloned().unwrap_or_default();

    let metrics = compute_metrics(&state, t, &from, &to, &status, &typ).await?;
    Ok(Json(serde_json::to_value(metrics).unwrap()))
}

#[derive(serde::Serialize)]
struct Metrics {
    rescue_volume: i64,
    adoption_conversion: f64,
    task_completion_rate: f64,
    donations_logged: i64,
    inventory_on_hand: i64,
    filters: serde_json::Value,
}

async fn compute_metrics(
    state: &AppState,
    t: &str,
    from: &str,
    to: &str,
    status_f: &str,
    type_f: &str,
) -> Result<Metrics, AppError> {
    // Base WHERE fragments — additive.
    let mut where_parts: Vec<String> = vec!["1=1".into()];
    let mut binds: Vec<String> = Vec::new();
    if !from.is_empty() { where_parts.push("created_at >= ?".into()); binds.push(from.into()); }
    if !to.is_empty() { where_parts.push("created_at <= ?".into()); binds.push(to.into()); }
    if !type_f.is_empty() { where_parts.push("intake_type = ?".into()); binds.push(type_f.into()); }
    let where_sql = where_parts.join(" AND ");

    // Helper to count intake rows matching filters plus an extra clause.
    async fn count_with(pool: &sqlx::SqlitePool, sql: String, binds: &[String], t: &str) -> Result<i64, AppError> {
        let mut q = sqlx::query_as::<_, (i64,)>(&sql);
        for b in binds { q = q.bind(b); }
        let (n,) = q.fetch_one(pool).await.map_err(db_err(t))?;
        Ok(n)
    }

    let intake_total = count_with(
        &state.db,
        format!("SELECT COUNT(*) FROM intake_records WHERE {}", where_sql),
        &binds, t,
    ).await?;

    let mut donations_binds = binds.clone();
    donations_binds.push("donation".into());
    let donations = count_with(
        &state.db,
        format!("SELECT COUNT(*) FROM intake_records WHERE {} AND intake_type = ?", where_sql),
        &donations_binds, t,
    ).await?;

    let mut animals_binds = binds.clone();
    animals_binds.push("animal".into());
    let animals = count_with(
        &state.db,
        format!("SELECT COUNT(*) FROM intake_records WHERE {} AND intake_type = ?", where_sql),
        &animals_binds, t,
    ).await?;

    let mut adopted_binds = binds.clone();
    adopted_binds.push("adopted".into());
    let adopted = count_with(
        &state.db,
        format!("SELECT COUNT(*) FROM intake_records WHERE {} AND status = ?", where_sql),
        &adopted_binds, t,
    ).await?;

    let tasks_done: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks WHERE status = 'completed'")
        .fetch_one(&state.db).await.map_err(db_err(t))?;
    let tasks_open: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks WHERE status IN ('open','in_progress')")
        .fetch_one(&state.db).await.map_err(db_err(t))?;
    let inventory: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM supply_entries")
        .fetch_one(&state.db).await.map_err(db_err(t))?;

    let adoption_conversion = if animals > 0 { adopted as f64 / animals as f64 } else { 0.0 };
    let task_completion_rate = if tasks_done.0 + tasks_open.0 > 0 {
        tasks_done.0 as f64 / (tasks_done.0 + tasks_open.0) as f64
    } else { 0.0 };

    Ok(Metrics {
        rescue_volume: intake_total,
        adoption_conversion,
        task_completion_rate,
        donations_logged: donations,
        inventory_on_hand: inventory.0,
        filters: serde_json::json!({
            "from": from,
            "to": to,
            "status": status_f,
            "intake_type": type_f,
        }),
    })
}

pub async fn export_csv(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let t = &tid.0;
    require_admin_or_auditor(&user, t)?;

    let from = q.get("from").cloned().unwrap_or_default();
    let to = q.get("to").cloned().unwrap_or_default();
    let status = q.get("status").cloned().unwrap_or_default();
    let typ = q.get("intake_type").cloned().unwrap_or_default();
    let m = compute_metrics(&state, t, &from, &to, &status, &typ).await?;

    let csv = format!(
        "metric,value\nrescue_volume,{}\ndonations_logged,{}\ninventory_on_hand,{}\nadoption_conversion,{}\ntask_completion_rate,{}\n",
        m.rescue_volume, m.donations_logged, m.inventory_on_hand, m.adoption_conversion, m.task_completion_rate
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"));
    headers.insert("Content-Disposition", HeaderValue::from_static("attachment; filename=\"report.csv\""));
    Ok((StatusCode::OK, headers, csv))
}

pub async fn adoption_conversion(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
) -> Result<Json<serde_json::Value>, AppError> {
    let t = &tid.0;
    let animals: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM intake_records WHERE intake_type = 'animal'")
        .fetch_one(&state.db).await.map_err(db_err(t))?;
    let adopted: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM intake_records WHERE status = 'adopted'")
        .fetch_one(&state.db).await.map_err(db_err(t))?;
    let rate = if animals.0 > 0 { (adopted.0 as f64) / (animals.0 as f64) } else { 0.0 };
    Ok(Json(serde_json::json!({"total": animals.0, "adopted": adopted.0, "conversion_rate": rate})))
}
