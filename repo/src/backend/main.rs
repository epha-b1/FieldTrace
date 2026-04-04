mod app;
mod config;
mod crypto;
mod error;
mod extractors;
mod middleware;
mod modules;

use tracing_subscriber::{fmt, EnvFilter, prelude::*};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().json())
        .init();

    let config = config::Config::from_env();
    tracing::info!(port = config.port, "Starting FieldTrace server");

    let app = app::create_app(&config).await;

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    tracing::info!(addr = %addr, "Listening");
    axum::serve(listener, app).await.expect("Server error");
}
