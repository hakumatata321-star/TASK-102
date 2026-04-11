use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::report_definition::*;
use crate::models::scheduled_report::*;
use crate::rbac::guard::check_permission_for_request;

fn check_perm(auth: &crate::auth::jwt::Claims, code: &str, req: &HttpRequest, conn: &mut diesel::PgConnection)
    -> Result<crate::rbac::data_scope::PermissionContext, AppError> {
    check_permission_for_request(auth, code, req.method().as_str(), req.path(), conn)
}
use crate::schema::{report_definitions, scheduled_reports};

// ===================== Report Definition CRUD =====================

pub async fn create_definition(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<CreateReportDefinitionRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.create", &req, &mut conn)?;

    // Validate kpi_type
    if !KPI_TYPES.contains(&body.kpi_type.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid kpi_type '{}'. Valid types: {:?}",
            body.kpi_type, KPI_TYPES
        )));
    }

    validate_dimensions_and_filters(&body.kpi_type, &body.dimensions, &body.filters)?;

    let new = NewReportDefinition {
        name: body.name.clone(),
        description: body.description.clone(),
        kpi_type: body.kpi_type.clone(),
        dimensions: body.dimensions.clone(),
        filters: body.filters.clone(),
        chart_config: body.chart_config.clone(),
        created_by: auth.0.sub,
    };

    let report: ReportDefinition = diesel::insert_into(report_definitions::table)
        .values(&new)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(ReportDefinitionResponse::from(report)))
}

pub async fn list_definitions(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.read", &req, &mut conn)?;

    let results: Vec<ReportDefinition> = report_definitions::table
        .filter(report_definitions::is_active.eq(true))
        .select(ReportDefinition::as_select())
        .order(report_definitions::name.asc())
        .load(&mut conn)?;

    let responses: Vec<ReportDefinitionResponse> =
        results.into_iter().map(ReportDefinitionResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_definition(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let report_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.read", &req, &mut conn)?;

    let report: ReportDefinition = report_definitions::table
        .find(report_id)
        .select(ReportDefinition::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(ReportDefinitionResponse::from(report)))
}

pub async fn update_definition(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateReportDefinitionRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let report_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.update", &req, &mut conn)?;

    let existing: ReportDefinition = report_definitions::table
        .find(report_id)
        .select(ReportDefinition::as_select())
        .first(&mut conn)?;

    if let Some(ref kpi) = body.kpi_type {
        if !KPI_TYPES.contains(&kpi.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid kpi_type '{}'. Valid types: {:?}",
                kpi, KPI_TYPES
            )));
        }
    }

    let effective_kpi = body
        .kpi_type
        .as_deref()
        .unwrap_or(existing.kpi_type.as_str())
        .to_string();
    let effective_dimensions = body
        .dimensions
        .clone()
        .unwrap_or(existing.dimensions.clone());
    let effective_filters = body
        .filters
        .clone()
        .unwrap_or(existing.filters.clone());
    validate_dimensions_and_filters(&effective_kpi, &effective_dimensions, &effective_filters)?;

    let changeset = UpdateReportDefinition {
        name: body.name.clone(),
        description: body.description.clone(),
        kpi_type: body.kpi_type.clone(),
        dimensions: body.dimensions.clone(),
        filters: body.filters.clone(),
        chart_config: body.chart_config.clone(),
        is_active: body.is_active,
        updated_at: Utc::now(),
    };

    let report: ReportDefinition = diesel::update(report_definitions::table.find(report_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(ReportDefinitionResponse::from(report)))
}

pub async fn delete_definition(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let report_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.delete", &req, &mut conn)?;

    diesel::update(report_definitions::table.find(report_id))
        .set((
            report_definitions::is_active.eq(false),
            report_definitions::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}

// ===================== Run Report (KPI Query) =====================

pub async fn run_report(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<RunReportRequest>,
) -> Result<HttpResponse, AppError> {
    let report_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.read", &req, &mut conn)?;

    let report: ReportDefinition = report_definitions::table
        .find(report_id)
        .select(ReportDefinition::as_select())
        .first(&mut conn)?;

    // Execute KPI query based on type.
    // Each KPI type returns aggregated data from existing tables.
    // The dimensions/filters from the report definition and the runtime
    // request are merged to build the query.
    let result = execute_kpi_query(&report, &body, &mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "report_id": report_id,
        "kpi_type": report.kpi_type,
        "dimensions": report.dimensions,
        "filters_applied": body.filters,
        "date_from": body.date_from,
        "date_to": body.date_to,
        "data": result,
        "generated_at": Utc::now(),
    })))
}

/// Supported dimensions per KPI type for validation.
fn valid_dimensions_for_kpi(kpi: &str) -> &[&str] {
    match kpi {
        "participation_by_store" => &["location"],
        "participation_by_department" => &["department"],
        "award_distribution" => &["location"],
        "registration_conversion" => &["department", "location"],
        "review_efficiency" => &[],
        "project_milestones" => &[],
        _ => &[],
    }
}

/// Supported filter keys per KPI type.
fn valid_filters_for_kpi(kpi: &str) -> &[&str] {
    match kpi {
        "participation_by_store" | "participation_by_department" => &["location", "department"],
        "award_distribution" => &["location"],
        "registration_conversion" => &["department", "location"],
        "review_efficiency" => &[],
        "project_milestones" => &[],
        _ => &[],
    }
}

fn validate_dimensions_and_filters(
    kpi: &str,
    dimensions: &serde_json::Value,
    filters: &serde_json::Value,
) -> Result<(), AppError> {
    let valid_dims = valid_dimensions_for_kpi(kpi);
    if let Some(dims) = dimensions.as_array() {
        for d in dims {
            if let Some(s) = d.as_str() {
                if !valid_dims.contains(&s) && !valid_dims.is_empty() {
                    return Err(AppError::Validation(format!(
                        "Unsupported dimension '{}' for KPI '{}'. Valid: {:?}",
                        s, kpi, valid_dims
                    )));
                }
            }
        }
    }

    let valid_fkeys = valid_filters_for_kpi(kpi);
    if let Some(obj) = filters.as_object() {
        for k in obj.keys() {
            if !valid_fkeys.contains(&k.as_str()) && !valid_fkeys.is_empty() {
                return Err(AppError::Validation(format!(
                    "Unsupported filter '{}' for KPI '{}'. Valid: {:?}",
                    k, kpi, valid_fkeys
                )));
            }
        }
    }

    Ok(())
}

fn execute_kpi_query(
    report: &ReportDefinition,
    request: &RunReportRequest,
    conn: &mut PgConnection,
) -> Result<serde_json::Value, AppError> {
    let date_from = request.date_from
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let date_to = request.date_to.unwrap_or_else(Utc::now);

    // Merge report definition filters with runtime filters (runtime overrides)
    let merged_filters = merge_filters(&report.filters, &request.filters);

    // Validate dimensions
    let valid_dims = valid_dimensions_for_kpi(&report.kpi_type);
    if let Some(dims) = report.dimensions.as_array() {
        for d in dims {
            if let Some(s) = d.as_str() {
                if !valid_dims.contains(&s) && !valid_dims.is_empty() {
                    return Err(AppError::Validation(format!(
                        "Unsupported dimension '{}' for KPI '{}'. Valid: {:?}",
                        s, report.kpi_type, valid_dims
                    )));
                }
            }
        }
    }

    // Validate filter keys
    let valid_fkeys = valid_filters_for_kpi(&report.kpi_type);
    if let Some(obj) = merged_filters.as_object() {
        for k in obj.keys() {
            if !valid_fkeys.contains(&k.as_str()) && !valid_fkeys.is_empty() {
                return Err(AppError::Validation(format!(
                    "Unsupported filter '{}' for KPI '{}'. Valid: {:?}",
                    k, report.kpi_type, valid_fkeys
                )));
            }
        }
    }

    // Extract commonly used filters
    let filter_location = merged_filters.get("location").and_then(|v| v.as_str());
    let filter_department = merged_filters.get("department").and_then(|v| v.as_str());

    match report.kpi_type.as_str() {
        "registration_conversion" => {
            use crate::schema::users;
            let mut q = users::table
                .filter(users::created_at.ge(date_from))
                .filter(users::created_at.le(date_to))
                .into_boxed();
            if let Some(dept) = filter_department {
                q = q.filter(users::department.eq(dept));
            }
            if let Some(loc) = filter_location {
                q = q.filter(users::location.eq(loc));
            }
            let total: i64 = q.count().get_result(conn)?;

            let mut qa = users::table
                .filter(users::created_at.ge(date_from))
                .filter(users::created_at.le(date_to))
                .filter(users::is_active.eq(true))
                .into_boxed();
            if let Some(dept) = filter_department {
                qa = qa.filter(users::department.eq(dept));
            }
            if let Some(loc) = filter_location {
                qa = qa.filter(users::location.eq(loc));
            }
            let active: i64 = qa.count().get_result(conn)?;

            let rate = if total > 0 { (active as f64 / total as f64) * 100.0 } else { 0.0 };
            Ok(serde_json::json!({
                "total_registrations": total,
                "active_users": active,
                "conversion_rate_pct": rate,
            }))
        }
        "participation_by_store" => {
            use crate::schema::participants;
            // Load matching participants then group in Rust (Diesel boxed doesn't support group_by)
            let all: Vec<crate::models::participant::Participant> = participants::table
                .filter(participants::created_at.ge(date_from))
                .filter(participants::created_at.le(date_to))
                .filter(participants::is_active.eq(true))
                .select(crate::models::participant::Participant::as_select())
                .load(conn)?;
            let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            for p in &all {
                if let Some(ref fl) = filter_location { if p.location.as_deref() != Some(*fl) { continue; } }
                if let Some(ref fd) = filter_department { if p.department.as_deref() != Some(*fd) { continue; } }
                let key = p.location.clone().unwrap_or_else(|| "unassigned".into());
                *counts.entry(key).or_insert(0) += 1;
            }
            let data: Vec<serde_json::Value> = counts.into_iter()
                .map(|(loc, cnt)| serde_json::json!({"location": loc, "participant_count": cnt}))
                .collect();
            Ok(serde_json::json!(data))
        }
        "participation_by_department" => {
            use crate::schema::participants;
            let all: Vec<crate::models::participant::Participant> = participants::table
                .filter(participants::created_at.ge(date_from))
                .filter(participants::created_at.le(date_to))
                .filter(participants::is_active.eq(true))
                .select(crate::models::participant::Participant::as_select())
                .load(conn)?;
            let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            for p in &all {
                if let Some(ref fl) = filter_location { if p.location.as_deref() != Some(*fl) { continue; } }
                if let Some(ref fd) = filter_department { if p.department.as_deref() != Some(*fd) { continue; } }
                let key = p.department.clone().unwrap_or_else(|| "unassigned".into());
                *counts.entry(key).or_insert(0) += 1;
            }
            let data: Vec<serde_json::Value> = counts.into_iter()
                .map(|(dept, cnt)| serde_json::json!({"department": dept, "participant_count": cnt}))
                .collect();
            Ok(serde_json::json!(data))
        }
        "project_milestones" => {
            use crate::schema::dataset_versions;
            let total_versions: i64 = dataset_versions::table
                .filter(dataset_versions::created_at.ge(date_from))
                .filter(dataset_versions::created_at.le(date_to))
                .count().get_result(conn)?;
            let current_versions: i64 = dataset_versions::table
                .filter(dataset_versions::is_current.eq(true))
                .count().get_result(conn)?;
            Ok(serde_json::json!({"versions_created_in_period": total_versions, "current_active_versions": current_versions}))
        }
        "review_efficiency" => {
            use crate::models::approval::ApprovalStatus;
            use crate::schema::approval_requests;
            let total: i64 = approval_requests::table.filter(approval_requests::created_at.ge(date_from)).filter(approval_requests::created_at.le(date_to)).count().get_result(conn)?;
            let approved: i64 = approval_requests::table.filter(approval_requests::created_at.ge(date_from)).filter(approval_requests::created_at.le(date_to)).filter(approval_requests::status.eq(ApprovalStatus::Approved)).count().get_result(conn)?;
            let rejected: i64 = approval_requests::table.filter(approval_requests::created_at.ge(date_from)).filter(approval_requests::created_at.le(date_to)).filter(approval_requests::status.eq(ApprovalStatus::Rejected)).count().get_result(conn)?;
            let pending: i64 = approval_requests::table.filter(approval_requests::created_at.ge(date_from)).filter(approval_requests::created_at.le(date_to)).filter(approval_requests::status.eq(ApprovalStatus::Pending)).count().get_result(conn)?;
            Ok(serde_json::json!({"total_reviews": total, "approved": approved, "rejected": rejected, "pending": pending, "approval_rate_pct": if total > 0 { (approved as f64 / total as f64) * 100.0 } else { 0.0 }}))
        }
        "award_distribution" => {
            use crate::schema::orders;
            let all: Vec<crate::models::order::Order> = orders::table
                .filter(orders::created_at.ge(date_from))
                .filter(orders::created_at.le(date_to))
                .select(crate::models::order::Order::as_select())
                .load(conn)?;
            let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            for o in &all {
                if let Some(ref fl) = filter_location { if o.location.as_str() != *fl { continue; } }
                *counts.entry(o.location.clone()).or_insert(0) += 1;
            }
            let data: Vec<serde_json::Value> = counts.into_iter()
                .map(|(loc, cnt)| serde_json::json!({"location": loc, "order_count": cnt}))
                .collect();
            Ok(serde_json::json!(data))
        }
        _ => Err(AppError::Validation(format!("Unknown kpi_type: {}", report.kpi_type))),
    }
}

/// Merge report-definition filters with runtime request filters.
/// Runtime values override definition values for the same key.
fn merge_filters(definition: &serde_json::Value, runtime: &serde_json::Value) -> serde_json::Value {
    let mut merged = serde_json::Map::new();
    if let Some(def) = definition.as_object() {
        for (k, v) in def {
            merged.insert(k.clone(), v.clone());
        }
    }
    if let Some(rt) = runtime.as_object() {
        for (k, v) in rt {
            merged.insert(k.clone(), v.clone());
        }
    }
    serde_json::Value::Object(merged)
}

// ===================== Scheduled Reports =====================

pub async fn create_schedule(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<CreateScheduledReportRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.schedule", &req, &mut conn)?;

    // Validate format
    validate_export_format(&body.export_format)?;

    // Verify report definition exists
    let _: ReportDefinition = report_definitions::table
        .find(body.report_definition_id)
        .select(ReportDefinition::as_select())
        .first(&mut conn)?;

    let new = NewScheduledReport {
        report_definition_id: body.report_definition_id,
        frequency: body.frequency.clone(),
        export_format: body.export_format.clone(),
        next_run_at: body.next_run_at,
        created_by: auth.0.sub,
    };

    let schedule: ScheduledReport = diesel::insert_into(scheduled_reports::table)
        .values(&new)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(ScheduledReportResponse::from(schedule)))
}

pub async fn list_schedules(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.schedule", &req, &mut conn)?;

    let results: Vec<ScheduledReport> = scheduled_reports::table
        .filter(scheduled_reports::is_active.eq(true))
        .select(ScheduledReport::as_select())
        .order(scheduled_reports::next_run_at.asc())
        .load(&mut conn)?;

    let responses: Vec<ScheduledReportResponse> =
        results.into_iter().map(ScheduledReportResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_schedule(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let schedule_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.schedule", &req, &mut conn)?;

    let schedule: ScheduledReport = scheduled_reports::table
        .find(schedule_id)
        .select(ScheduledReport::as_select())
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(ScheduledReportResponse::from(schedule)))
}

pub async fn update_schedule(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateScheduledReportRequest>,
) -> Result<HttpResponse, AppError> {
    let schedule_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.schedule", &req, &mut conn)?;

    if let Some(ref fmt) = body.export_format {
        validate_export_format(fmt)?;
    }

    let changeset = UpdateScheduledReport {
        frequency: body.frequency.clone(),
        export_format: body.export_format.clone(),
        next_run_at: body.next_run_at,
        is_active: body.is_active,
        updated_at: Utc::now(),
    };

    let schedule: ScheduledReport = diesel::update(scheduled_reports::table.find(schedule_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(ScheduledReportResponse::from(schedule)))
}

pub async fn delete_schedule(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let schedule_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.schedule", &req, &mut conn)?;

    diesel::update(scheduled_reports::table.find(schedule_id))
        .set((
            scheduled_reports::is_active.eq(false),
            scheduled_reports::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}

// ===================== KPI types listing =====================

pub async fn list_kpi_types(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "report.read", &req, &mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "kpi_types": KPI_TYPES,
    })))
}

fn validate_export_format(fmt: &str) -> Result<(), AppError> {
    match fmt {
        "xlsx" | "pdf" | "csv" => Ok(()),
        _ => Err(AppError::Validation(format!(
            "Invalid export format '{}'. Allowed: xlsx, pdf, csv",
            fmt
        ))),
    }
}
