//! Idempotency middleware.
//!
//! Scope = (method, normalized_route, actor_id, Idempotency-Key).
//! Window = 10 minutes (records older than that are ignored and cleaned up).
//!
//! Flow:
//!  1. Middleware runs AFTER session middleware so `SessionUser` is known
//!     (auth-first scope per requirements).
//!  2. If request has an `Idempotency-Key` header, look up existing record.
//!  3. On hit (same method+route+actor_id+key and < 10 min old): replay the
//!     stored status + body.
//!  4. On miss: run the handler, buffer the response body, persist it, then
//!     forward to the client.
//!
//! The middleware is applied to the shared protected router; handlers that
//! don't want idempotency simply don't send the header.
//!
//! Normalized route = the matched route path (parameters not substituted).

use axum::body::{to_bytes, Body};
use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use crate::app::AppState;
use crate::extractors::SessionUser;

const MAX_BODY_BYTES: usize = 1024 * 1024; // 1 MiB replay cap

pub async fn idempotency_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // Only touch mutating methods
    let method = request.method().clone();
    if !matches!(method.as_str(), "POST" | "PATCH" | "PUT" | "DELETE") {
        return next.run(request).await;
    }

    // Require an authenticated actor (auth-first)
    let actor_id = match request.extensions().get::<SessionUser>() {
        Some(u) => u.user_id.clone(),
        None => return next.run(request).await,
    };

    // Read header
    let key = match request.headers().get("Idempotency-Key") {
        Some(v) => match v.to_str() {
            Ok(s) if !s.is_empty() && s.len() <= 128 => s.to_string(),
            _ => return next.run(request).await,
        },
        None => return next.run(request).await,
    };

    // Normalized route comes from axum's matched path extension
    let route = request
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    // Lookup existing record within 10 min
    let existing: Option<(i64, String, i64)> = sqlx::query_as(
        "SELECT id, response_body, status_code \
         FROM idempotency_keys \
         WHERE method = ? AND route = ? AND actor_id = ? AND key_value = ? \
           AND created_at > datetime('now', '-10 minutes')",
    )
    .bind(method.as_str())
    .bind(&route)
    .bind(&actor_id)
    .bind(&key)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some((_id, body, status_code)) = existing {
        // Replay — outer trace_id middleware will add X-Trace-Id on the way out.
        let status = StatusCode::from_u16(status_code as u16).unwrap_or(StatusCode::OK);
        let resp = Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .header("Idempotent-Replay", HeaderValue::from_static("true"))
            .body(Body::from(body))
            .unwrap();
        return resp;
    }

    // Cache miss — run handler, then buffer body
    let response = next.run(request).await;
    let (parts, body) = response.into_parts();

    // Only store JSON-ish successful responses (status < 500). We do NOT store
    // 5xx responses because the next retry should be allowed to succeed.
    let status_code = parts.status.as_u16() as i64;
    let bytes_res = to_bytes(body, MAX_BODY_BYTES).await;
    let bytes = match bytes_res {
        Ok(b) => b,
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };

    if status_code < 500 {
        let body_str = String::from_utf8_lossy(&bytes).into_owned();
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO idempotency_keys (method, route, actor_id, key_value, response_body, status_code) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(method.as_str())
        .bind(&route)
        .bind(&actor_id)
        .bind(&key)
        .bind(&body_str)
        .bind(status_code)
        .execute(&state.db)
        .await;

        // Also clean up stale rows (older than 10 min) opportunistically.
        let _ = sqlx::query(
            "DELETE FROM idempotency_keys WHERE created_at < datetime('now', '-10 minutes')",
        )
        .execute(&state.db)
        .await;
    }

    Response::from_parts(parts, Body::from(bytes))
}
