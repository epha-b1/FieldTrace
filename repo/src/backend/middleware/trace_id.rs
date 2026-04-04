use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;
use uuid::Uuid;

/// Attaches a unique trace ID to every request.
/// - Sets `X-Trace-Id` response header
/// - Creates a tracing span with trace_id, method, and URI
pub async fn trace_id_middleware(mut request: Request, next: Next) -> Response {
    let trace_id = Uuid::new_v4().to_string();

    // Store trace_id in request extensions for downstream use (e.g. error responses)
    request.extensions_mut().insert(TraceId(trace_id.clone()));

    let span = tracing::info_span!(
        "request",
        trace_id = %trace_id,
        method = %request.method(),
        uri = %request.uri(),
    );

    async move {
        tracing::info!("Request started");
        let mut response = next.run(request).await;
        response.headers_mut().insert(
            "X-Trace-Id",
            HeaderValue::from_str(&trace_id).unwrap(),
        );
        tracing::info!(status = %response.status(), "Request completed");
        response
    }
    .instrument(span)
    .await
}

/// Trace ID stored in request extensions, accessible from handlers.
#[derive(Clone, Debug)]
pub struct TraceId(pub String);
