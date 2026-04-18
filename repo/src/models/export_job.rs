use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::export_jobs;

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[ExistingTypePath = "crate::schema::sql_types::ExportStatusType"]
pub enum ExportStatus {
    #[db_rename = "queued"]
    Queued,
    #[db_rename = "running"]
    Running,
    #[db_rename = "completed"]
    Completed,
    #[db_rename = "failed"]
    Failed,
    #[db_rename = "cancelled"]
    Cancelled,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = export_jobs)]
pub struct ExportJob {
    pub id: Uuid,
    pub report_definition_id: Uuid,
    pub export_format: String,
    pub status: ExportStatus,
    pub total_rows: Option<i64>,
    pub processed_rows: i64,
    pub progress_pct: i16,
    pub file_path: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub error_message: Option<String>,
    pub approval_request_id: Option<Uuid>,
    pub requested_by: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub sha256_hash: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = export_jobs)]
pub struct NewExportJob {
    pub report_definition_id: Uuid,
    pub export_format: String,
    pub status: ExportStatus,
    pub total_rows: Option<i64>,
    pub approval_request_id: Option<Uuid>,
    pub requested_by: Uuid,
}

#[derive(Serialize)]
pub struct ExportJobResponse {
    pub id: Uuid,
    pub report_definition_id: Uuid,
    pub export_format: String,
    pub status: ExportStatus,
    pub total_rows: Option<i64>,
    pub processed_rows: i64,
    pub progress_pct: i16,
    pub file_path: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub error_message: Option<String>,
    pub approval_request_id: Option<Uuid>,
    pub requested_by: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub sha256_hash: Option<String>,
}

impl From<ExportJob> for ExportJobResponse {
    fn from(e: ExportJob) -> Self {
        Self {
            id: e.id,
            report_definition_id: e.report_definition_id,
            export_format: e.export_format,
            status: e.status,
            total_rows: e.total_rows,
            processed_rows: e.processed_rows,
            progress_pct: e.progress_pct,
            file_path: e.file_path,
            file_size_bytes: e.file_size_bytes,
            error_message: e.error_message,
            approval_request_id: e.approval_request_id,
            requested_by: e.requested_by,
            started_at: e.started_at,
            completed_at: e.completed_at,
            created_at: e.created_at,
            sha256_hash: e.sha256_hash,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateExportRequest {
    pub report_definition_id: Uuid,
    #[serde(default = "default_format")]
    pub export_format: String,
    pub estimated_rows: Option<i64>,
}

fn default_format() -> String {
    "xlsx".into()
}

#[derive(Deserialize)]
pub struct ExportQueryParams {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// Max rows before requiring bulk export approval.
pub const BULK_EXPORT_THRESHOLD: i64 = 250_000;
