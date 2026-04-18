use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::approval::NewApprovalRequest;
use crate::models::export_job::*;
use crate::models::report_definition::ReportDefinition;
use crate::rbac::guard::{check_permission, check_permission_for_request, check_permission_no_approval, resolve_permission_id};
use crate::schema::{approval_requests, export_jobs, report_definitions};

fn check_perm_req(
    auth: &crate::auth::jwt::Claims, code: &str, req: &HttpRequest, conn: &mut diesel::PgConnection,
) -> Result<crate::rbac::data_scope::PermissionContext, AppError> {
    check_permission_for_request(auth, code, req.method().as_str(), req.path(), conn)
}

pub async fn request_export(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<CreateExportRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm_req(&auth.0, "report.export", &req, &mut conn)?;

    // Validate format
    validate_format(&body.export_format)?;

    // Verify report exists
    let _report: ReportDefinition = report_definitions::table
        .find(body.report_definition_id)
        .select(ReportDefinition::as_select())
        .first(&mut conn)?;

    let estimated = body.estimated_rows.unwrap_or(0);
    let needs_approval = estimated > BULK_EXPORT_THRESHOLD;

    let approval_id = if needs_approval {
        // Check that user has the bulk export permission
        check_permission_no_approval(&auth.0, "report.export.bulk", &mut conn)?;

        let perm_id = resolve_permission_id("report.export.bulk", &mut conn)?;
        let payload = serde_json::json!({
            "type": "bulk_export",
            "report_definition_id": body.report_definition_id,
            "export_format": body.export_format,
            "estimated_rows": estimated,
            "requested_by": auth.0.sub,
        });

        let approval: crate::models::approval::ApprovalRequest =
            diesel::insert_into(approval_requests::table)
                .values(&NewApprovalRequest {
                    permission_point_id: perm_id,
                    requester_user_id: auth.0.sub,
                    payload,
                })
                .returning(crate::models::approval::ApprovalRequest::as_returning())
                .get_result(&mut conn)?;

        Some(approval.id)
    } else {
        None
    };

    let new = NewExportJob {
        report_definition_id: body.report_definition_id,
        export_format: body.export_format.clone(),
        status: if needs_approval {
            ExportStatus::Queued
        } else {
            ExportStatus::Queued
        },
        total_rows: body.estimated_rows,
        approval_request_id: approval_id,
        requested_by: auth.0.sub,
    };

    let job: ExportJob = diesel::insert_into(export_jobs::table)
        .values(&new)
        .get_result(&mut conn)?;

    if needs_approval {
        Ok(HttpResponse::Accepted().json(serde_json::json!({
            "export_job": ExportJobResponse::from(job),
            "message": "Bulk export requires approval before processing",
            "approval_request_id": approval_id,
        })))
    } else {
        // Job stays Queued — the async export worker picks it up
        // autonomously and transitions it through Running → Completed/Failed.
        let updated: ExportJob = export_jobs::table
            .find(job.id)
            .select(ExportJob::as_select())
            .first(&mut conn)?;

        Ok(HttpResponse::Accepted().json(ExportJobResponse::from(updated)))
    }
}

pub async fn get_job(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let job_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm_req(&auth.0, "report.export", &req, &mut conn)?;

    let job: ExportJob = export_jobs::table
        .find(job_id)
        .select(ExportJob::as_select())
        .first(&mut conn)?;

    // Owner or admin check
    if job.requested_by != auth.0.sub {
        check_permission(&auth.0, "report.export.admin", &mut conn)?;
    }

    Ok(HttpResponse::Ok().json(ExportJobResponse::from(job)))
}

pub async fn list_jobs(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    query: web::Query<ExportQueryParams>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm_req(&auth.0, "report.export", &req, &mut conn)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let mut q = export_jobs::table
        .filter(export_jobs::requested_by.eq(auth.0.sub))
        .into_boxed();

    if let Some(ref status) = query.status {
        q = q.filter(export_jobs::status.eq(parse_export_status(status)?));
    }

    let results: Vec<ExportJob> = q
        .select(ExportJob::as_select())
        .order(export_jobs::created_at.desc())
        .offset(offset)
        .limit(per_page)
        .load(&mut conn)?;

    let responses: Vec<ExportJobResponse> =
        results.into_iter().map(ExportJobResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn admin_list_jobs(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    query: web::Query<ExportQueryParams>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm_req(&auth.0, "report.export.admin", &req, &mut conn)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let mut q = export_jobs::table.into_boxed();

    if let Some(ref status) = query.status {
        q = q.filter(export_jobs::status.eq(parse_export_status(status)?));
    }

    let results: Vec<ExportJob> = q
        .select(ExportJob::as_select())
        .order(export_jobs::created_at.desc())
        .offset(offset)
        .limit(per_page)
        .load(&mut conn)?;

    let responses: Vec<ExportJobResponse> =
        results.into_iter().map(ExportJobResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn update_progress(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateProgressRequest>,
) -> Result<HttpResponse, AppError> {
    let job_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm_req(&auth.0, "report.export.admin", &req, &mut conn)?;

    let job: ExportJob = export_jobs::table
        .find(job_id)
        .select(ExportJob::as_select())
        .first(&mut conn)?;

    if job.status != ExportStatus::Running {
        return Err(AppError::Validation(
            "Can only update progress of running jobs".into(),
        ));
    }

    let pct = if let Some(total) = job.total_rows {
        if total > 0 {
            ((body.processed_rows as f64 / total as f64) * 100.0) as i16
        } else {
            0
        }
    } else {
        0
    };

    diesel::update(export_jobs::table.find(job_id))
        .set((
            export_jobs::processed_rows.eq(body.processed_rows),
            export_jobs::progress_pct.eq(pct.min(100)),
        ))
        .execute(&mut conn)?;

    let updated: ExportJob = export_jobs::table
        .find(job_id)
        .select(ExportJob::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(ExportJobResponse::from(updated)))
}

/// Complete an export job. The caller provides total_rows and file_content
/// (base64-encoded). The server stores the artifact in managed storage,
/// computes SHA-256, and records the metadata. Never trusts caller-provided paths.
pub async fn complete_job(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<CompleteJobRequest>,
) -> Result<HttpResponse, AppError> {
    let job_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm_req(&auth.0, "report.export.admin", &req, &mut conn)?;

    let job: ExportJob = export_jobs::table
        .find(job_id)
        .select(ExportJob::as_select())
        .first(&mut conn)?;

    if job.status == ExportStatus::Completed {
        return Ok(HttpResponse::Ok().json(ExportJobResponse::from(job)));
    }
    if job.status != ExportStatus::Queued && job.status != ExportStatus::Running {
        return Err(AppError::Validation(
            "Can only complete queued or running jobs".into(),
        ));
    }

    // Decode file content from base64 (if provided)
    let file_bytes = if let Some(ref b64) = body.file_content_base64 {
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
            .map_err(|_| AppError::Validation("Invalid base64 file content".into()))?
    } else {
        // If no content provided, store an empty artifact as placeholder
        Vec::new()
    };

    crate::storage::validate_file_size(file_bytes.len() as u64)?;

    // Server-managed storage — never accept caller paths
    let (managed_path, sha256) = crate::storage::save_artifact(
        "exports", job_id, &job.export_format, &file_bytes,
    )?;

    let actual_size = file_bytes.len() as i64;

    let now = Utc::now();
    diesel::update(export_jobs::table.find(job_id))
        .set((
            export_jobs::status.eq(ExportStatus::Completed),
            export_jobs::processed_rows.eq(body.total_rows),
            export_jobs::total_rows.eq(Some(body.total_rows)),
            export_jobs::progress_pct.eq(100i16),
            export_jobs::file_path.eq(&managed_path),
            export_jobs::file_size_bytes.eq(Some(actual_size)),
            export_jobs::sha256_hash.eq(Some(&sha256)),
            export_jobs::started_at.eq(job.started_at.or(Some(now))),
            export_jobs::completed_at.eq(Some(now)),
        ))
        .execute(&mut conn)?;

    let updated: ExportJob = export_jobs::table
        .find(job_id)
        .select(ExportJob::as_select())
        .first(&mut conn)?;

    let after = serde_json::json!({"job_id": job_id, "sha256": sha256, "size": actual_size});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "create", "export_artifact", Some(job_id), None, Some(&after));

    Ok(HttpResponse::Ok().json(ExportJobResponse::from(updated)))
}

pub async fn fail_job(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<FailJobRequest>,
) -> Result<HttpResponse, AppError> {
    let job_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm_req(&auth.0, "report.export.admin", &req, &mut conn)?;

    let before = serde_json::json!({"job_id": job_id, "status": "running"});
    diesel::update(export_jobs::table.find(job_id))
        .set((
            export_jobs::status.eq(ExportStatus::Failed),
            export_jobs::error_message.eq(&body.error_message),
            export_jobs::completed_at.eq(Some(Utc::now())),
        ))
        .execute(&mut conn)?;

    let after = serde_json::json!({"job_id": job_id, "status": "failed"});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "update", "export_jobs", Some(job_id), Some(&before), Some(&after));

    let updated: ExportJob = export_jobs::table
        .find(job_id)
        .select(ExportJob::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(ExportJobResponse::from(updated)))
}

pub async fn cancel_job(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    _req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let job_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    check_permission_no_approval(&auth.0, "report.export", &mut conn)?;

    let job: ExportJob = export_jobs::table
        .find(job_id)
        .select(ExportJob::as_select())
        .first(&mut conn)?;

    // Only the requester or admin can cancel, and only queued/running jobs
    if job.requested_by != auth.0.sub {
        check_permission(&auth.0, "report.export.admin", &mut conn)?;
    }

    if job.status != ExportStatus::Queued && job.status != ExportStatus::Running {
        return Err(AppError::Validation(
            "Can only cancel queued or running jobs".into(),
        ));
    }

    diesel::update(export_jobs::table.find(job_id))
        .set((
            export_jobs::status.eq(ExportStatus::Cancelled),
            export_jobs::completed_at.eq(Some(Utc::now())),
        ))
        .execute(&mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "cancelled" })))
}

pub async fn download_export(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let job_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm_req(&auth.0, "report.export.download", &req, &mut conn)?;

    let job: ExportJob = export_jobs::table
        .find(job_id)
        .select(ExportJob::as_select())
        .first(&mut conn)?;

    // Owner or admin check
    if job.requested_by != auth.0.sub {
        check_permission(&auth.0, "report.export.admin", &mut conn)?;
    }

    if job.status != ExportStatus::Completed {
        return Err(AppError::Validation("Export is not yet completed".into()));
    }

    let file_path = job.file_path.ok_or_else(|| {
        AppError::Internal("Completed export has no file path".into())
    })?;

    let data = crate::storage::read_file(&file_path)?;

    let content_type = match job.export_format.as_str() {
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pdf" => "application/pdf",
        "csv" => "text/csv",
        _ => "application/octet-stream",
    };

    let filename = format!("export-{}.{}", job_id, job.export_format);

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(data))
}

// ===================== Request DTOs =====================

#[derive(serde::Deserialize)]
pub struct UpdateProgressRequest {
    pub processed_rows: i64,
}

#[derive(serde::Deserialize)]
pub struct CompleteJobRequest {
    pub total_rows: i64,
    /// Base64-encoded file content for the export artifact.
    /// Server manages storage path — no caller-provided paths accepted.
    pub file_content_base64: Option<String>,
}

use base64;

#[derive(serde::Deserialize)]
pub struct FailJobRequest {
    pub error_message: String,
}

// ===================== Helpers =====================

fn validate_format(fmt: &str) -> Result<(), AppError> {
    match fmt {
        "xlsx" | "pdf" | "csv" => Ok(()),
        _ => Err(AppError::Validation(format!(
            "Invalid export format '{}'. Allowed: xlsx, pdf, csv",
            fmt
        ))),
    }
}

fn parse_export_status(s: &str) -> Result<ExportStatus, AppError> {
    match s {
        "queued" => Ok(ExportStatus::Queued),
        "running" => Ok(ExportStatus::Running),
        "completed" => Ok(ExportStatus::Completed),
        "failed" => Ok(ExportStatus::Failed),
        "cancelled" => Ok(ExportStatus::Cancelled),
        _ => Err(AppError::Validation(format!("Invalid export status: {}", s))),
    }
}
