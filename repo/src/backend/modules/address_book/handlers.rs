use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use uuid::Uuid;

use crate::app::AppState;
use crate::crypto;
use crate::error::AppError;
use crate::extractors::SessionUser;
use crate::middleware::trace_id::TraceId;
use fieldtrace_shared::*;

fn validate_zip(zip: &str, tid: &str) -> Result<(), AppError> {
    let re = zip.len() == 10
        && zip.chars().take(5).all(|c| c.is_ascii_digit())
        && zip.chars().nth(5) == Some('-')
        && zip.chars().skip(6).all(|c| c.is_ascii_digit());
    if !re {
        return Err(AppError::validation("ZIP+4 must be NNNNN-NNNN format", tid));
    }
    Ok(())
}

pub async fn list(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
) -> Result<Json<Vec<AddressResponse>>, AppError> {
    let rows = sqlx::query_as::<_, AddrRow>(
        "SELECT id, label, street_enc, city_enc, state_enc, zip_plus4, phone_enc, created_at \
         FROM address_book WHERE user_id = ? ORDER BY created_at",
    )
    .bind(&user.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::internal(e.to_string(), ""))?;

    let c = crypto::Crypto::new(&state.config.encryption_key);
    Ok(Json(rows.into_iter().map(|r| to_response(&c, r)).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Json(body): Json<AddressRequest>,
) -> Result<(StatusCode, Json<AddressResponse>), AppError> {
    let t = &tid.0;
    validate_zip(&body.zip_plus4, t)?;

    let c = crypto::Crypto::new(&state.config.encryption_key);
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO address_book (id, user_id, label, street_enc, city_enc, state_enc, zip_plus4, phone_enc) \
         VALUES (?,?,?,?,?,?,?,?)",
    )
    .bind(&id).bind(&user.user_id).bind(&body.label)
    .bind(c.encrypt(&body.street)).bind(c.encrypt(&body.city))
    .bind(c.encrypt(&body.state)).bind(&body.zip_plus4)
    .bind(c.encrypt(&body.phone))
    .execute(&state.db).await
    .map_err(|e| AppError::internal(e.to_string(), t))?;

    Ok((StatusCode::CREATED, Json(AddressResponse {
        id, label: body.label, street: body.street, city: body.city,
        state: body.state, zip_plus4: body.zip_plus4,
        phone_masked: crypto::mask_phone(&body.phone), created_at: String::new(),
    })))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Path(addr_id): Path<String>,
    Json(body): Json<AddressRequest>,
) -> Result<Json<AddressResponse>, AppError> {
    let t = &tid.0;
    validate_zip(&body.zip_plus4, t)?;

    let c = crypto::Crypto::new(&state.config.encryption_key);
    let res = sqlx::query(
        "UPDATE address_book SET label=?, street_enc=?, city_enc=?, state_enc=?, zip_plus4=?, phone_enc=?, updated_at=datetime('now') \
         WHERE id=? AND user_id=?",
    )
    .bind(&body.label).bind(c.encrypt(&body.street)).bind(c.encrypt(&body.city))
    .bind(c.encrypt(&body.state)).bind(&body.zip_plus4).bind(c.encrypt(&body.phone))
    .bind(&addr_id).bind(&user.user_id)
    .execute(&state.db).await
    .map_err(|e| AppError::internal(e.to_string(), t))?;

    if res.rows_affected() == 0 {
        return Err(AppError::not_found("Address not found or not yours", t));
    }

    Ok(Json(AddressResponse {
        id: addr_id, label: body.label, street: body.street, city: body.city,
        state: body.state, zip_plus4: body.zip_plus4,
        phone_masked: crypto::mask_phone(&body.phone), created_at: String::new(),
    }))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(tid): Extension<TraceId>,
    Extension(user): Extension<SessionUser>,
    Path(addr_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let t = &tid.0;
    let res = sqlx::query("DELETE FROM address_book WHERE id=? AND user_id=?")
        .bind(&addr_id).bind(&user.user_id)
        .execute(&state.db).await
        .map_err(|e| AppError::internal(e.to_string(), t))?;

    if res.rows_affected() == 0 {
        return Err(AppError::not_found("Address not found or not yours", t));
    }
    Ok(Json(serde_json::json!({"message":"Deleted"})))
}

fn to_response(c: &crypto::Crypto, r: AddrRow) -> AddressResponse {
    let phone = c.decrypt(&r.phone_enc);
    AddressResponse {
        id: r.id, label: r.label,
        street: c.decrypt(&r.street_enc),
        city: c.decrypt(&r.city_enc),
        state: c.decrypt(&r.state_enc),
        zip_plus4: r.zip_plus4,
        phone_masked: crypto::mask_phone(&phone),
        created_at: r.created_at,
    }
}

#[derive(sqlx::FromRow)]
struct AddrRow {
    id: String, label: String,
    street_enc: String, city_enc: String, state_enc: String,
    zip_plus4: String, phone_enc: String, created_at: String,
}
