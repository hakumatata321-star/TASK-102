use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::approval::NewApprovalRequest;
use crate::models::dataset::*;
use crate::models::dataset_version::*;
use crate::models::field_dictionary::*;
use crate::models::version_lineage::*;
use crate::rbac::guard::{check_permission_for_request, check_permission_no_approval, resolve_permission_id};
use crate::schema::{
    approval_requests, dataset_versions, datasets, field_dictionaries, version_lineage,
};

fn check_perm(auth: &crate::auth::jwt::Claims, code: &str, req: &HttpRequest, conn: &mut diesel::PgConnection)
    -> Result<crate::rbac::data_scope::PermissionContext, AppError> {
    check_permission_for_request(auth, code, req.method().as_str(), req.path(), conn)
}

// ===================== Dataset CRUD =====================

pub async fn create_dataset(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<CreateDatasetRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.create", &req, &mut conn)?;

    let new = NewDataset {
        name: body.name.clone(),
        description: body.description.clone(),
        dataset_type: body.dataset_type.clone(),
        created_by: auth.0.sub,
    };

    let dataset: Dataset = diesel::insert_into(datasets::table)
        .values(&new)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(DatasetResponse::from(dataset)))
}

pub async fn list_datasets(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    query: web::Query<DatasetQueryParams>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.read", &req, &mut conn)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let mut q = datasets::table.into_boxed();

    if let Some(active) = query.is_active {
        q = q.filter(datasets::is_active.eq(active));
    } else {
        q = q.filter(datasets::is_active.eq(true));
    }

    if let Some(ref dt) = query.dataset_type {
        q = q.filter(datasets::dataset_type.eq(parse_dataset_type(dt)?));
    }

    let results: Vec<Dataset> = q
        .select(Dataset::as_select())
        .order(datasets::name.asc())
        .offset(offset)
        .limit(per_page)
        .load(&mut conn)?;

    // Attach current version number to each dataset
    let responses: Vec<DatasetResponse> = results
        .into_iter()
        .map(|d| {
            let mut resp = DatasetResponse::from(d.clone());
            let current_ver: Option<i32> = dataset_versions::table
                .filter(dataset_versions::dataset_id.eq(d.id))
                .filter(dataset_versions::is_current.eq(true))
                .select(dataset_versions::version_number)
                .first(&mut conn)
                .optional()
                .unwrap_or(None);
            resp.current_version = current_ver;
            resp
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_dataset(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let dataset_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.read", &req, &mut conn)?;

    let dataset: Dataset = datasets::table
        .find(dataset_id)
        .select(Dataset::as_select())
        .first(&mut conn)?;

    let current_ver: Option<i32> = dataset_versions::table
        .filter(dataset_versions::dataset_id.eq(dataset_id))
        .filter(dataset_versions::is_current.eq(true))
        .select(dataset_versions::version_number)
        .first(&mut conn)
        .optional()?;

    let mut resp = DatasetResponse::from(dataset);
    resp.current_version = current_ver;
    Ok(HttpResponse::Ok().json(resp))
}

pub async fn update_dataset(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateDatasetRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let dataset_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.update", &req, &mut conn)?;

    let changeset = UpdateDataset {
        name: body.name.clone(),
        description: body.description.clone(),
        dataset_type: body.dataset_type.clone(),
        is_active: body.is_active,
        updated_at: Utc::now(),
    };

    let dataset: Dataset = diesel::update(datasets::table.find(dataset_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(DatasetResponse::from(dataset)))
}

pub async fn deactivate_dataset(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let dataset_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.delete", &req, &mut conn)?;

    diesel::update(datasets::table.find(dataset_id))
        .set((
            datasets::is_active.eq(false),
            datasets::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    Ok(HttpResponse::NoContent().finish())
}

// ===================== Versions =====================

pub async fn create_version(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<CreateVersionRequest>,
) -> Result<HttpResponse, AppError> {
    let dataset_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.version.create", &req, &mut conn)?;

    // Verify dataset exists and is active
    let _dataset: Dataset = datasets::table
        .find(dataset_id)
        .filter(datasets::is_active.eq(true))
        .select(Dataset::as_select())
        .first(&mut conn)
        .map_err(|_| AppError::NotFound("Dataset not found or inactive".into()))?;

    let result = conn.transaction::<_, AppError, _>(|conn| {
        // Determine next version number
        let max_ver: Option<i32> = dataset_versions::table
            .filter(dataset_versions::dataset_id.eq(dataset_id))
            .select(diesel::dsl::max(dataset_versions::version_number))
            .first(conn)?;
        let next_ver = max_ver.unwrap_or(0) + 1;

        // Mark all existing versions as not current
        diesel::update(
            dataset_versions::table
                .filter(dataset_versions::dataset_id.eq(dataset_id))
                .filter(dataset_versions::is_current.eq(true)),
        )
        .set(dataset_versions::is_current.eq(false))
        .execute(conn)?;

        // Insert the new version
        let new = NewDatasetVersion {
            dataset_id,
            version_number: next_ver,
            storage_path: body.storage_path.clone(),
            file_size_bytes: body.file_size_bytes,
            sha256_hash: body.sha256_hash.clone(),
            row_count: body.row_count,
            transformation_note: body.transformation_note.clone(),
            is_current: true,
            created_by: auth.0.sub,
        };

        let version: DatasetVersion = diesel::insert_into(dataset_versions::table)
            .values(&new)
            .get_result(conn)?;

        // Insert lineage links
        for parent_id in &body.parent_version_ids {
            let lineage = NewVersionLineage {
                child_version_id: version.id,
                parent_version_id: *parent_id,
            };
            diesel::insert_into(version_lineage::table)
                .values(&lineage)
                .execute(conn)?;
        }

        // Insert field dictionary entries
        for fd in &body.field_dictionary {
            let new_fd = NewFieldDictionary {
                version_id: version.id,
                field_name: fd.field_name.clone(),
                field_type: fd.field_type.clone(),
                meaning: fd.meaning.clone(),
                source_system: fd.source_system.clone(),
            };
            diesel::insert_into(field_dictionaries::table)
                .values(&new_fd)
                .execute(conn)?;
        }

        // Update dataset timestamp
        diesel::update(datasets::table.find(dataset_id))
            .set(datasets::updated_at.eq(Utc::now()))
            .execute(conn)?;

        Ok(version)
    })?;

    let mut resp = DatasetVersionResponse::from(result.clone());
    resp.parent_version_ids = Some(body.parent_version_ids.clone());

    // Attach field dictionary
    let fields: Vec<FieldDictionary> = field_dictionaries::table
        .filter(field_dictionaries::version_id.eq(result.id))
        .select(FieldDictionary::as_select())
        .load(&mut conn)?;
    resp.field_dictionary = Some(fields.into_iter().map(FieldDictionaryResponse::from).collect());

    Ok(HttpResponse::Created().json(resp))
}

pub async fn list_versions(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    query: web::Query<VersionQueryParams>,
) -> Result<HttpResponse, AppError> {
    let dataset_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.version.read", &req, &mut conn)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let versions: Vec<DatasetVersion> = dataset_versions::table
        .filter(dataset_versions::dataset_id.eq(dataset_id))
        .select(DatasetVersion::as_select())
        .order(dataset_versions::version_number.desc())
        .offset(offset)
        .limit(per_page)
        .load(&mut conn)?;

    let responses: Vec<DatasetVersionResponse> =
        versions.into_iter().map(DatasetVersionResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_version(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (_dataset_id, version_id) = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.version.read", &req, &mut conn)?;

    let version: DatasetVersion = dataset_versions::table
        .find(version_id)
        .select(DatasetVersion::as_select())
        .first(&mut conn)?;

    let parents: Vec<Uuid> = version_lineage::table
        .filter(version_lineage::child_version_id.eq(version_id))
        .select(version_lineage::parent_version_id)
        .load(&mut conn)?;

    let fields: Vec<FieldDictionary> = field_dictionaries::table
        .filter(field_dictionaries::version_id.eq(version_id))
        .select(FieldDictionary::as_select())
        .order(field_dictionaries::field_name.asc())
        .load(&mut conn)?;

    let mut resp = DatasetVersionResponse::from(version);
    resp.parent_version_ids = Some(parents);
    resp.field_dictionary = Some(fields.into_iter().map(FieldDictionaryResponse::from).collect());

    Ok(HttpResponse::Ok().json(resp))
}

// ===================== Lineage =====================

pub async fn get_lineage(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (_dataset_id, version_id) = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.version.read", &req, &mut conn)?;

    // Parents (what this version was derived from)
    let parents: Vec<VersionLineage> = version_lineage::table
        .filter(version_lineage::child_version_id.eq(version_id))
        .select(VersionLineage::as_select())
        .load(&mut conn)?;

    // Children (versions derived from this one)
    let children: Vec<VersionLineage> = version_lineage::table
        .filter(version_lineage::parent_version_id.eq(version_id))
        .select(VersionLineage::as_select())
        .load(&mut conn)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "version_id": version_id,
        "parents": parents.into_iter().map(LineageResponse::from).collect::<Vec<_>>(),
        "children": children.into_iter().map(LineageResponse::from).collect::<Vec<_>>(),
    })))
}

// ===================== Rollback =====================

pub async fn rollback(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<RollbackRequest>,
) -> Result<HttpResponse, AppError> {
    let dataset_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;

    // dataset.rollback has requires_approval = TRUE, so check_permission will
    // reject if the user doesn't hold it. The approval workflow is triggered
    // separately. Here we use check_permission_no_approval so the handler can
    // create the approval request itself with context.
    let _ctx = check_permission_no_approval(&auth.0, "dataset.rollback", &mut conn)?;

    // Verify target version exists and belongs to this dataset
    let target: DatasetVersion = dataset_versions::table
        .find(body.target_version_id)
        .select(DatasetVersion::as_select())
        .first(&mut conn)
        .map_err(|_| AppError::NotFound("Target version not found".into()))?;

    if target.dataset_id != dataset_id {
        return Err(AppError::Validation(
            "Target version does not belong to this dataset".into(),
        ));
    }

    // Create approval request for this rollback
    let perm_id = resolve_permission_id("dataset.rollback", &mut conn)?;
    let payload = serde_json::json!({
        "type": "dataset_rollback",
        "dataset_id": dataset_id,
        "target_version_id": body.target_version_id,
        "target_version_number": target.version_number,
        "note": body.note,
        "requested_by": auth.0.sub,
    });

    let approval = NewApprovalRequest {
        permission_point_id: perm_id,
        requester_user_id: auth.0.sub,
        payload,
    };

    let approval_req: crate::models::approval::ApprovalRequest =
        diesel::insert_into(approval_requests::table)
            .values(&approval)
            .returning(crate::models::approval::ApprovalRequest::as_returning())
            .get_result(&mut conn)?;

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "message": "Rollback requires approval",
        "approval_request_id": approval_req.id,
        "dataset_id": dataset_id,
        "target_version_id": body.target_version_id,
        "target_version_number": target.version_number,
    })))
}

pub async fn execute_rollback(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    _req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<ExecuteRollbackRequest>,
) -> Result<HttpResponse, AppError> {
    let dataset_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission_no_approval(&auth.0, "dataset.rollback", &mut conn)?;

    // Verify the approval request is approved
    let approval: crate::models::approval::ApprovalRequest = approval_requests::table
        .find(body.approval_request_id)
        .select(crate::models::approval::ApprovalRequest::as_select())
        .first(&mut conn)
        .map_err(|_| AppError::NotFound("Approval request not found".into()))?;

    if approval.status != crate::models::approval::ApprovalStatus::Approved {
        return Err(AppError::Validation(
            "Approval request must be approved before executing rollback".into(),
        ));
    }

    // Extract target from the approval payload
    let target_version_id: Uuid = approval.payload["target_version_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::Internal("Invalid approval payload".into()))?;

    let target: DatasetVersion = dataset_versions::table
        .find(target_version_id)
        .select(DatasetVersion::as_select())
        .first(&mut conn)?;

    if target.dataset_id != dataset_id {
        return Err(AppError::Validation(
            "Target version does not belong to this dataset".into(),
        ));
    }

    // Capture before-state
    let current_ver: Option<i32> = dataset_versions::table
        .filter(dataset_versions::dataset_id.eq(dataset_id))
        .filter(dataset_versions::is_current.eq(true))
        .select(dataset_versions::version_number)
        .first(&mut conn)
        .optional()?;
    let before = serde_json::json!({"dataset_id": dataset_id, "current_version": current_ver, "action": "rollback"});

    // Perform the rollback in a transaction
    let rolled_back = conn.transaction::<_, AppError, _>(|conn| {
        // Unmark all current versions
        diesel::update(
            dataset_versions::table
                .filter(dataset_versions::dataset_id.eq(dataset_id))
                .filter(dataset_versions::is_current.eq(true)),
        )
        .set(dataset_versions::is_current.eq(false))
        .execute(conn)?;

        // Mark the target version as current
        diesel::update(dataset_versions::table.find(target_version_id))
            .set(dataset_versions::is_current.eq(true))
            .execute(conn)?;

        // Update dataset timestamp
        diesel::update(datasets::table.find(dataset_id))
            .set(datasets::updated_at.eq(Utc::now()))
            .execute(conn)?;

        // Re-fetch the version
        let v: DatasetVersion = dataset_versions::table
            .find(target_version_id)
            .select(DatasetVersion::as_select())
            .first(conn)?;
        Ok(v)
    })?;

    let after = serde_json::json!({"dataset_id": dataset_id, "rolled_back_to_version": target.version_number, "action": "rollback"});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "update", "dataset_versions", Some(dataset_id), Some(&before), Some(&after));

    let mut resp = DatasetVersionResponse::from(rolled_back);

    let fields: Vec<FieldDictionary> = field_dictionaries::table
        .filter(field_dictionaries::version_id.eq(target_version_id))
        .select(FieldDictionary::as_select())
        .load(&mut conn)?;
    resp.field_dictionary = Some(fields.into_iter().map(FieldDictionaryResponse::from).collect());

    Ok(HttpResponse::Ok().json(resp))
}

#[derive(serde::Deserialize)]
pub struct ExecuteRollbackRequest {
    pub approval_request_id: Uuid,
}

// ===================== Field Dictionary =====================

pub async fn list_field_dictionary(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (_dataset_id, version_id) = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.version.read", &req, &mut conn)?;

    let fields: Vec<FieldDictionary> = field_dictionaries::table
        .filter(field_dictionaries::version_id.eq(version_id))
        .select(FieldDictionary::as_select())
        .order(field_dictionaries::field_name.asc())
        .load(&mut conn)?;

    let responses: Vec<FieldDictionaryResponse> =
        fields.into_iter().map(FieldDictionaryResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn add_field_entry(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
    body: web::Json<FieldDictionaryInput>,
) -> Result<HttpResponse, AppError> {
    let (_dataset_id, version_id) = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.field_dict.manage", &req, &mut conn)?;

    let new = NewFieldDictionary {
        version_id,
        field_name: body.field_name.clone(),
        field_type: body.field_type.clone(),
        meaning: body.meaning.clone(),
        source_system: body.source_system.clone(),
    };

    let field: FieldDictionary = diesel::insert_into(field_dictionaries::table)
        .values(&new)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Created().json(FieldDictionaryResponse::from(field)))
}

pub async fn update_field_entry(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<(Uuid, Uuid, Uuid)>,
    body: web::Json<UpdateFieldDictionaryRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let (_dataset_id, _version_id, field_id) = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.field_dict.manage", &req, &mut conn)?;

    let changeset = UpdateFieldDictionary {
        field_type: body.field_type.clone(),
        meaning: body.meaning.clone(),
        source_system: body.source_system.clone(),
        last_updated_at: Utc::now(),
    };

    let field: FieldDictionary = diesel::update(field_dictionaries::table.find(field_id))
        .set(&changeset)
        .get_result(&mut conn)?;

    Ok(HttpResponse::Ok().json(FieldDictionaryResponse::from(field)))
}

pub async fn delete_field_entry(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<(Uuid, Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (_dataset_id, _version_id, field_id) = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm(&auth.0, "dataset.field_dict.manage", &req, &mut conn)?;

    diesel::delete(field_dictionaries::table.find(field_id)).execute(&mut conn)?;
    Ok(HttpResponse::NoContent().finish())
}

// ===================== Helpers =====================

fn parse_dataset_type(s: &str) -> Result<DatasetType, AppError> {
    match s {
        "raw" => Ok(DatasetType::Raw),
        "cleaned" => Ok(DatasetType::Cleaned),
        "feature" => Ok(DatasetType::Feature),
        "result" => Ok(DatasetType::Result),
        _ => Err(AppError::Validation(format!("Invalid dataset type: {}", s))),
    }
}
