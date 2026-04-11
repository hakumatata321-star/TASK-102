use actix_web::HttpResponse;
use chrono::Utc;
use diesel::prelude::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::idempotency::{IdempotencyRecord, NewIdempotencyRecord};
use crate::schema::idempotency_keys;

/// Derive a resource-scoped idempotency key to avoid cross-endpoint key collisions.
pub fn scoped_key(key: Uuid, resource_type: &str) -> Uuid {
    let mut hasher = Sha256::new();
    hasher.update(resource_type.as_bytes());
    hasher.update(b":");
    hasher.update(key.as_bytes());
    let out = hasher.finalize();
    Uuid::from_slice(&out[..16]).unwrap_or(key)
}

/// Check if an idempotency key has already been used.
/// Returns `Some(HttpResponse)` with the cached response if found, `None` if new.
pub fn check_idempotency(
    key: Uuid,
    conn: &mut PgConnection,
) -> Result<Option<HttpResponse>, AppError> {
    let record: Option<IdempotencyRecord> = idempotency_keys::table
        .find(key)
        .select(IdempotencyRecord::as_select())
        .first(conn)
        .optional()?;

    match record {
        Some(rec) => {
            if rec.expires_at < Utc::now() {
                // Expired — treat as new
                diesel::delete(idempotency_keys::table.find(key)).execute(conn)?;
                Ok(None)
            } else {
                // Return cached response
                let status = actix_web::http::StatusCode::from_u16(rec.response_status as u16)
                    .unwrap_or(actix_web::http::StatusCode::OK);
                Ok(Some(HttpResponse::build(status).json(rec.response_body)))
            }
        }
        None => Ok(None),
    }
}

/// Atomically reserve an idempotency key inside a DB transaction.
/// Uses INSERT with PK constraint to prevent concurrent duplicate mutations.
/// Returns Ok(None) if reserved (proceed with mutation).
/// Returns Ok(Some(response)) if already completed (replay cached).
pub fn reserve_idempotency_key(
    key: Uuid,
    resource_type: &str,
    conn: &mut PgConnection,
) -> Result<Option<HttpResponse>, AppError> {
    // Check for existing completed record first
    let existing: Option<IdempotencyRecord> = idempotency_keys::table
        .find(key)
        .select(IdempotencyRecord::as_select())
        .first(conn)
        .optional()?;

    if let Some(rec) = existing {
        if rec.expires_at < Utc::now() {
            diesel::delete(idempotency_keys::table.find(key)).execute(conn)?;
        } else if rec.response_status > 0 {
            let status = actix_web::http::StatusCode::from_u16(rec.response_status as u16)
                .unwrap_or(actix_web::http::StatusCode::OK);
            return Ok(Some(HttpResponse::build(status).json(rec.response_body)));
        } else {
            // Sentinel 0 = another transaction in progress
            return Err(AppError::Conflict(
                "Duplicate request in progress for this idempotency key".into(),
            ));
        }
    }

    // Reserve atomically via PK insert (sentinel status=0)
    let placeholder = NewIdempotencyRecord {
        key,
        resource_type: resource_type.to_string(),
        resource_id: Uuid::nil(),
        response_status: 0,
        response_body: serde_json::json!(null),
    };

    let inserted = diesel::insert_into(idempotency_keys::table)
        .values(&placeholder)
        .on_conflict_do_nothing()
        .execute(conn)?;

    if inserted == 0 {
        // Lost race — another tx inserted first
        return Err(AppError::Conflict(
            "Duplicate request for this idempotency key".into(),
        ));
    }

    Ok(None)
}

/// Finalize a reserved idempotency key with actual response after mutation.
pub fn finalize_idempotency(
    key: Uuid,
    resource_id: Uuid,
    status: i16,
    body: &serde_json::Value,
    conn: &mut PgConnection,
) -> Result<(), AppError> {
    diesel::update(idempotency_keys::table.find(key))
        .set((
            idempotency_keys::resource_id.eq(resource_id),
            idempotency_keys::response_status.eq(status),
            idempotency_keys::response_body.eq(body),
        ))
        .execute(conn)?;
    Ok(())
}

/// Store idempotency response (upsert). Used for paths where
/// reservation + finalization is not yet adopted.
pub fn store_idempotency(
    key: Uuid,
    resource_type: &str,
    resource_id: Uuid,
    status: i16,
    body: &serde_json::Value,
    conn: &mut PgConnection,
) -> Result<(), AppError> {
    let new_record = NewIdempotencyRecord {
        key,
        resource_type: resource_type.to_string(),
        resource_id,
        response_status: status,
        response_body: body.clone(),
    };

    diesel::insert_into(idempotency_keys::table)
        .values(&new_record)
        .on_conflict(idempotency_keys::key)
        .do_update()
        .set((
            idempotency_keys::resource_id.eq(resource_id),
            idempotency_keys::response_status.eq(status),
            idempotency_keys::response_body.eq(body),
        ))
        .execute(conn)?;

    Ok(())
}
