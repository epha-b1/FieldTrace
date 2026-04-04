use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::AppError;
use crate::extractors::SessionUser;
use crate::middleware::trace_id::TraceId;

fn tid(req: &Request) -> String {
    req.extensions()
        .get::<TraceId>()
        .map(|t| t.0.clone())
        .unwrap_or_default()
}

/// Rejects with 401 if no valid session.
pub async fn require_auth(request: Request, next: Next) -> Response {
    if request.extensions().get::<SessionUser>().is_some() {
        next.run(request).await
    } else {
        AppError::unauthorized("Authentication required", tid(&request)).into_response()
    }
}

/// Rejects with 401 if no session, 403 if not administrator.
pub async fn require_admin(request: Request, next: Next) -> Response {
    match request.extensions().get::<SessionUser>() {
        Some(u) if u.role == "administrator" => next.run(request).await,
        Some(_) => {
            AppError::forbidden("Administrator access required", tid(&request)).into_response()
        }
        None => {
            AppError::unauthorized("Authentication required", tid(&request)).into_response()
        }
    }
}
