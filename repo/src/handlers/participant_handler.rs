use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::participant::*;
use crate::models::tag::*;
use crate::rbac::guard::{check_permission_for_request, check_permission_no_approval};
use crate::schema::{participant_tags, participants, tags};

fn check_perm(auth: &crate::auth::jwt::Claims, code: &str, req: &HttpRequest, conn: &mut diesel::PgConnection)
    -> Result<crate::rbac::data_scope::PermissionContext, AppError> {
    check_permission_for_request(auth, code, req.method().as_str(), req.path(), conn)
}

pub async fn create(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<CreateParticipantRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "participant.create", &req, &mut conn)?;

    let (participant, tag_names) = conn.transaction::<_, AppError, _>(|conn| {
        let new = NewParticipant {
            first_name: body.first_name.clone(),
            last_name: body.last_name.clone(),
            email: body.email.clone(),
            phone: body.phone.clone(),
            department: body.department.clone(),
            location: body.location.clone(),
            employee_id: body.employee_id.clone(),
            notes: body.notes.clone(),
            created_by: auth.0.sub,
        };

        let participant: Participant = diesel::insert_into(participants::table)
            .values(&new)
            .get_result(conn)?;

        let tag_names = apply_tags(conn, participant.id, &body.tags)?;
        Ok((participant, tag_names))
    })?;

    let after = serde_json::json!({"id": participant.id, "name": format!("{} {}", participant.first_name, participant.last_name)});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "create", "participants", Some(participant.id), None, Some(&after));

    let mut resp = ParticipantResponse::from(participant);
    resp.tags = Some(tag_names);
    Ok(HttpResponse::Created().json(resp))
}

pub async fn list(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    query: web::Query<ParticipantSearchParams>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm(&auth.0, "participant.read", &req, &mut conn)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let mut q = participants::table.into_boxed();

    // Data-scope
    match ctx.data_scope.as_str() {
        "department" => {
            if let Some(ref dept) = ctx.department {
                q = q.filter(participants::department.eq(dept));
            }
        }
        "location" => {
            if let Some(ref loc) = ctx.location {
                q = q.filter(participants::location.eq(loc));
            }
        }
        "individual" => {
            q = q.filter(participants::created_by.eq(ctx.user_id));
        }
        _ => {}
    }

    // Filters
    if let Some(ref dept) = query.department {
        q = q.filter(participants::department.eq(dept));
    }
    if let Some(ref loc) = query.location {
        q = q.filter(participants::location.eq(loc));
    }
    if let Some(active) = query.is_active {
        q = q.filter(participants::is_active.eq(active));
    } else {
        q = q.filter(participants::is_active.eq(true));
    }

    // Text search on name/email/employee_id
    if let Some(ref search) = query.q {
        let pattern = format!("%{}%", search);
        q = q.filter(
            participants::first_name
                .ilike(pattern.clone())
                .or(participants::last_name.ilike(pattern.clone()))
                .or(participants::email.ilike(pattern.clone()))
                .or(participants::employee_id.ilike(pattern)),
        );
    }

    // Tag filter: find participant IDs that have the given tag
    if let Some(ref tag_name) = query.tag {
        let tagged_ids: Vec<Uuid> = participant_tags::table
            .inner_join(tags::table)
            .filter(tags::name.eq(tag_name))
            .select(participant_tags::participant_id)
            .load(&mut conn)?;
        q = q.filter(participants::id.eq_any(tagged_ids));
    }

    let results: Vec<Participant> = q
        .select(Participant::as_select())
        .order((participants::last_name.asc(), participants::first_name.asc()))
        .offset(offset)
        .limit(per_page)
        .load(&mut conn)?;

    let responses: Vec<ParticipantResponse> =
        results.into_iter().map(ParticipantResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let pid = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm(&auth.0, "participant.read", &req, &mut conn)?;

    let participant: Participant = participants::table
        .find(pid)
        .select(Participant::as_select())
        .first(&mut conn)?;

    ctx.enforce_scope(
        participant.created_by,
        participant.department.as_deref(),
        participant.location.as_deref(),
    )?;

    let tag_names = get_tag_names(&mut conn, pid)?;

    let mut resp = ParticipantResponse::from(participant);
    resp.tags = Some(tag_names);
    Ok(HttpResponse::Ok().json(resp))
}

pub async fn update(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateParticipantRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let pid = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm(&auth.0, "participant.update", &req, &mut conn)?;

    let existing: Participant = participants::table.find(pid).select(Participant::as_select()).first(&mut conn)?;
    ctx.enforce_scope(existing.created_by, existing.department.as_deref(), existing.location.as_deref())?;

    let changeset = UpdateParticipant {
        first_name: body.first_name.clone(),
        last_name: body.last_name.clone(),
        email: body.email.clone(),
        phone: body.phone.clone(),
        department: body.department.clone(),
        location: body.location.clone(),
        employee_id: body.employee_id.clone(),
        notes: body.notes.clone(),
        is_active: body.is_active,
        updated_at: Utc::now(),
    };

    let before = serde_json::json!({"id": existing.id, "name": format!("{} {}", existing.first_name, existing.last_name), "is_active": existing.is_active});

    let updated: Participant = diesel::update(participants::table.find(pid))
        .set(&changeset)
        .get_result(&mut conn)?;

    let after = serde_json::json!({"id": updated.id, "name": format!("{} {}", updated.first_name, updated.last_name), "is_active": updated.is_active});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "update", "participants", Some(pid), Some(&before), Some(&after));

    Ok(HttpResponse::Ok().json(ParticipantResponse::from(updated)))
}

pub async fn deactivate(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let pid = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm(&auth.0, "participant.delete", &req, &mut conn)?;
    let p: Participant = participants::table.find(pid).select(Participant::as_select()).first(&mut conn)?;
    ctx.enforce_scope(p.created_by, p.department.as_deref(), p.location.as_deref())?;

    diesel::update(participants::table.find(pid))
        .set((
            participants::is_active.eq(false),
            participants::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}

// --- Tags ---

pub async fn set_tags(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<SetTagsRequest>,
) -> Result<HttpResponse, AppError> {
    let pid = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm(&auth.0, "participant.tag", &req, &mut conn)?;
    let p: Participant = participants::table.find(pid).select(Participant::as_select()).first(&mut conn)?;
    ctx.enforce_scope(p.created_by, p.department.as_deref(), p.location.as_deref())?;

    let tag_names = conn.transaction::<_, AppError, _>(|conn| {
        // Remove existing tags
        diesel::delete(
            participant_tags::table.filter(participant_tags::participant_id.eq(pid)),
        )
        .execute(conn)?;

        apply_tags(conn, pid, &body.tags)
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "tags": tag_names })))
}

pub async fn get_tags(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    _req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let pid = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission_no_approval(&auth.0, "participant.read", &mut conn)?;
    let p: Participant = participants::table.find(pid).select(Participant::as_select()).first(&mut conn)?;
    ctx.enforce_scope(p.created_by, p.department.as_deref(), p.location.as_deref())?;

    let names = get_tag_names(&mut conn, pid)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "tags": names })))
}

// --- Bulk operations ---

pub async fn bulk_tag(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<BulkTagRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm(&auth.0, "participant.bulk", &req, &mut conn)?;

    // Validate each target is in caller scope — fail fast on first violation
    for pid in &body.participant_ids {
        let p: Participant = participants::table.find(*pid).select(Participant::as_select()).first(&mut conn)?;
        ctx.enforce_scope(p.created_by, p.department.as_deref(), p.location.as_deref())?;
    }

    let mut affected = 0usize;
    conn.transaction::<_, AppError, _>(|conn| {
        for pid in &body.participant_ids {
            let applied = apply_tags(conn, *pid, &body.tags)?;
            if !applied.is_empty() {
                affected += 1;
            }
        }
        Ok(())
    })?;

    let after = serde_json::json!({"action": "bulk_tag", "targets": body.participant_ids.len(), "tags": &body.tags});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "update", "participant_tags", None, None, Some(&after));

    Ok(HttpResponse::Ok().json(BulkResultResponse { affected }))
}

pub async fn bulk_deactivate(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<BulkDeactivateRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm(&auth.0, "participant.bulk", &req, &mut conn)?;

    // Validate each target is in caller scope — fail fast
    for pid in &body.participant_ids {
        let p: Participant = participants::table.find(*pid).select(Participant::as_select()).first(&mut conn)?;
        ctx.enforce_scope(p.created_by, p.department.as_deref(), p.location.as_deref())?;
    }

    let before = serde_json::json!({"targets": body.participant_ids.len(), "action": "bulk_deactivate"});

    let affected = diesel::update(
        participants::table.filter(participants::id.eq_any(&body.participant_ids)),
    )
    .set((
        participants::is_active.eq(false),
        participants::updated_at.eq(Utc::now()),
    ))
    .execute(&mut conn)?;

    let after = serde_json::json!({"targets": body.participant_ids.len(), "affected": affected, "action": "bulk_deactivate"});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "delete", "participants", None, Some(&before), Some(&after));

    Ok(HttpResponse::Ok().json(BulkResultResponse { affected }))
}

// --- Helpers ---

/// Ensure tags exist (create if needed) and link them to a participant.
/// Returns the list of tag names that were applied.
fn apply_tags(
    conn: &mut PgConnection,
    participant_id: Uuid,
    tag_names: &[String],
) -> Result<Vec<String>, AppError> {
    let mut result = Vec::new();
    for name in tag_names {
        let trimmed = name.trim().to_lowercase();
        if trimmed.is_empty() {
            continue;
        }

        // Upsert tag
        let tag: Tag = diesel::insert_into(tags::table)
            .values(&NewTag {
                name: trimmed.clone(),
            })
            .on_conflict(tags::name)
            .do_update()
            .set(tags::name.eq(&trimmed))
            .get_result(conn)?;

        // Link participant <-> tag (ignore duplicate)
        diesel::insert_into(participant_tags::table)
            .values(&NewParticipantTag {
                participant_id,
                tag_id: tag.id,
            })
            .on_conflict_do_nothing()
            .execute(conn)?;

        result.push(trimmed);
    }
    Ok(result)
}

fn get_tag_names(conn: &mut PgConnection, participant_id: Uuid) -> Result<Vec<String>, AppError> {
    let names: Vec<String> = participant_tags::table
        .inner_join(tags::table)
        .filter(participant_tags::participant_id.eq(participant_id))
        .select(tags::name)
        .order(tags::name.asc())
        .load(conn)?;
    Ok(names)
}
