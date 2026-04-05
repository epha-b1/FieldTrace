pub mod handlers;

use sqlx::SqlitePool;

/// Append-only audit log write. There is NO update/delete surface.
pub async fn write(
    db: &SqlitePool,
    actor_id: &str,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    trace_id: &str,
) {
    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_id, action, resource_type, resource_id, trace_id) VALUES (?,?,?,?,?)"
    )
    .bind(actor_id).bind(action).bind(resource_type).bind(resource_id).bind(trace_id)
    .execute(db).await;
}
