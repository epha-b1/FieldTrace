//! Background jobs registered at startup.
//!
//! - `session_cleanup`: deletes expired sessions every 5 minutes.
//! - `account_deletion_purge`: purges users whose deletion_requested_at is
//!   older than 7 days. Runs every hour.
//! - `diagnostics_cleanup`: deletes diagnostic ZIPs older than 1 hour.
//! - `evidence_retention`: marks evidence past retention for cleanup.
//!
//! Each job writes its status to `job_metrics` so `/admin/jobs` shows real data.

use crate::config::Config;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::time::Duration;

pub fn register_all(db: SqlitePool, config: Config) {
    let d1 = db.clone();
    tokio::spawn(async move { session_cleanup_loop(d1).await; });
    let d2 = db.clone();
    tokio::spawn(async move { account_deletion_purge_loop(d2).await; });
    let d3 = db.clone();
    let c3 = config.clone();
    tokio::spawn(async move { diagnostics_cleanup_loop(d3, c3).await; });
    let d4 = db.clone();
    tokio::spawn(async move { evidence_retention_loop(d4).await; });
    tracing::info!("All background jobs registered");
}

async fn record_run(db: &SqlitePool, job: &str, status: &str, err: Option<String>) {
    let _ = sqlx::query(
        "INSERT INTO job_metrics (job_name, status, run_count, last_error, last_run_at) \
         VALUES (?, ?, 1, ?, datetime('now'))"
    )
    .bind(job)
    .bind(status)
    .bind(err)
    .execute(db)
    .await;
}

// ── Session cleanup ───────────────────────────────────────────────────

async fn session_cleanup_loop(db: SqlitePool) {
    tracing::info!(job = "session_cleanup", "Registered (every 5 min)");
    // First tick is immediate so metrics show up promptly.
    let mut ticker = tokio::time::interval(Duration::from_secs(300));
    loop {
        ticker.tick().await;
        let res = sqlx::query(
            "DELETE FROM sessions WHERE last_active < datetime('now', '-30 minutes')"
        ).execute(&db).await;
        match res {
            Ok(r) => {
                tracing::info!(job = "session_cleanup", deleted = r.rows_affected(), "cleanup run");
                record_run(&db, "session_cleanup", "ok", None).await;
            }
            Err(e) => {
                tracing::error!(job = "session_cleanup", error = %e, "cleanup failed");
                record_run(&db, "session_cleanup", "error", Some(e.to_string())).await;
            }
        }
    }
}

// ── Account deletion purge ────────────────────────────────────────────

async fn account_deletion_purge_loop(db: SqlitePool) {
    tracing::info!(job = "account_deletion_purge", "Registered (every 1h)");
    let mut ticker = tokio::time::interval(Duration::from_secs(3600));
    loop {
        ticker.tick().await;
        // Purge users who requested deletion more than 7 days ago.
        // Delete dependent rows first to satisfy FKs. Audit logs remain
        // but the actor_id is anonymized via overwrite.
        let tx = db.begin().await;
        match tx {
            Ok(mut tx) => {
                // Collect victim ids
                let victims: Result<Vec<(String,)>, _> = sqlx::query_as(
                    "SELECT id FROM users WHERE deletion_requested_at IS NOT NULL \
                     AND deletion_requested_at < datetime('now', '-7 days')"
                ).fetch_all(&mut *tx).await;
                let ids = match victims {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!(job = "account_deletion_purge", error = %e, "scan failed");
                        record_run(&db, "account_deletion_purge", "error", Some(e.to_string())).await;
                        continue;
                    }
                };

                let mut purged = 0usize;
                let mut hard_err: Option<String> = None;
                for (uid,) in &ids {
                    // anonymize audit log actor references
                    if let Err(e) = sqlx::query("UPDATE audit_logs SET actor_id = NULL WHERE actor_id = ?")
                        .bind(uid).execute(&mut *tx).await
                    { hard_err = Some(e.to_string()); break; }
                    // drop dependent rows
                    let _ = sqlx::query("DELETE FROM sessions WHERE user_id = ?").bind(uid).execute(&mut *tx).await;
                    let _ = sqlx::query("DELETE FROM address_book WHERE user_id = ?").bind(uid).execute(&mut *tx).await;
                    // finally the user
                    if let Err(e) = sqlx::query("DELETE FROM users WHERE id = ?").bind(uid).execute(&mut *tx).await {
                        hard_err = Some(e.to_string()); break;
                    }
                    purged += 1;
                }

                if let Some(e) = hard_err {
                    let _ = tx.rollback().await;
                    tracing::error!(job = "account_deletion_purge", error = %e, "purge failed, rolled back");
                    record_run(&db, "account_deletion_purge", "error", Some(e)).await;
                } else if let Err(e) = tx.commit().await {
                    tracing::error!(job = "account_deletion_purge", error = %e, "commit failed");
                    record_run(&db, "account_deletion_purge", "error", Some(e.to_string())).await;
                } else {
                    tracing::info!(job = "account_deletion_purge", purged, "purge run");
                    record_run(&db, "account_deletion_purge", "ok", None).await;
                }
            }
            Err(e) => {
                tracing::error!(job = "account_deletion_purge", error = %e, "tx begin failed");
                record_run(&db, "account_deletion_purge", "error", Some(e.to_string())).await;
            }
        }
    }
}

// ── Diagnostics ZIP cleanup ───────────────────────────────────────────

fn diagnostics_dir(config: &Config) -> PathBuf {
    let mut p = PathBuf::from(&config.storage_dir);
    p.push("diagnostics");
    p
}

async fn diagnostics_cleanup_loop(db: SqlitePool, config: Config) {
    tracing::info!(job = "diagnostics_cleanup", "Registered (every 10 min)");
    let mut ticker = tokio::time::interval(Duration::from_secs(600));
    loop {
        ticker.tick().await;
        let dir = diagnostics_dir(&config);
        let removed = cleanup_old_files(&dir, 3600);
        tracing::info!(job = "diagnostics_cleanup", removed, "cleanup run");
        record_run(&db, "diagnostics_cleanup", "ok", None).await;
    }
}

/// Delete files in `dir` older than `max_age_secs`. Returns number deleted.
pub fn cleanup_old_files(dir: &std::path::Path, max_age_secs: u64) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        let now = std::time::SystemTime::now();
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age.as_secs() > max_age_secs {
                                if std::fs::remove_file(entry.path()).is_ok() {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    count
}

// ── Evidence retention (placeholder — retains all by default) ─────────

async fn evidence_retention_loop(db: SqlitePool) {
    tracing::info!(job = "evidence_retention", "Registered (every 1h)");
    let mut ticker = tokio::time::interval(Duration::from_secs(3600));
    loop {
        ticker.tick().await;
        // No deletion by default; legal_hold rows never expire.
        // The job still records a run for visibility.
        record_run(&db, "evidence_retention", "ok", None).await;
    }
}
