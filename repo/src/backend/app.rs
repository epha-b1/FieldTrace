use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use tower_http::services::ServeDir;

use crate::config::Config;
use crate::middleware::{auth_guard, session, trace_id};
use crate::modules::address_book::handlers as addr;
use crate::modules::auth::handlers as auth;
use crate::modules::users::handlers as users;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
}

pub async fn create_app(config: &Config) -> Router {
    let db = connect_db(&config.database_url).await;
    run_migrations(&db).await;

    let state = AppState {
        db,
        config: config.clone(),
    };

    // Public routes — no auth
    let public = Router::new()
        .route("/health", get(health_handler))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login));

    // Protected routes — valid session required
    let protected = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/auth/change-password", patch(auth::change_password))
        .route("/address-book", get(addr::list).post(addr::create))
        .route("/address-book/:id", patch(addr::update).delete(addr::delete))
        .layer(axum::middleware::from_fn(auth_guard::require_auth));

    // Admin routes — administrator role required
    let admin = Router::new()
        .route("/users", get(users::list_users).post(users::create_user))
        .route(
            "/users/:id",
            patch(users::update_user).delete(users::delete_user),
        )
        .layer(axum::middleware::from_fn(auth_guard::require_admin));

    Router::new()
        .merge(public)
        .merge(protected)
        .merge(admin)
        .with_state(state.clone())
        .fallback_service(ServeDir::new(&config.static_dir))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            session::session_middleware,
        ))
        .layer(axum::middleware::from_fn(trace_id::trace_id_middleware))
}

async fn connect_db(url: &str) -> SqlitePool {
    let options = SqliteConnectOptions::from_str(url)
        .expect("Invalid DATABASE_URL")
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Failed to connect to SQLite")
}

async fn run_migrations(db: &SqlitePool) {
    sqlx::migrate!("../../migrations")
        .run(db)
        .await
        .expect("Failed to run migrations");
    tracing::info!("Migrations applied successfully");
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}
