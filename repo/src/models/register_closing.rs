use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::register_closings;

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[ExistingTypePath = "crate::schema::sql_types::ClosingStatusType"]
pub enum ClosingStatus {
    #[db_rename = "pending"]
    Pending,
    #[db_rename = "confirmed"]
    Confirmed,
    #[db_rename = "variance_flagged"]
    VarianceFlagged,
    #[db_rename = "manager_confirmed"]
    ManagerConfirmed,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = register_closings)]
pub struct RegisterClosing {
    pub id: Uuid,
    pub location: String,
    pub cashier_user_id: Uuid,
    pub closing_date: NaiveDate,
    pub expected_cash_cents: i64,
    pub actual_cash_cents: i64,
    pub expected_card_cents: i64,
    pub actual_card_cents: i64,
    pub expected_gift_card_cents: i64,
    pub actual_gift_card_cents: i64,
    pub variance_cents: i64,
    pub status: ClosingStatus,
    pub approval_request_id: Option<Uuid>,
    pub notes: Option<String>,
    pub closed_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = register_closings)]
pub struct NewRegisterClosing {
    pub location: String,
    pub cashier_user_id: Uuid,
    pub closing_date: NaiveDate,
    pub expected_cash_cents: i64,
    pub actual_cash_cents: i64,
    pub expected_card_cents: i64,
    pub actual_card_cents: i64,
    pub expected_gift_card_cents: i64,
    pub actual_gift_card_cents: i64,
    pub variance_cents: i64,
    pub status: ClosingStatus,
    pub approval_request_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterClosingResponse {
    pub id: Uuid,
    pub location: String,
    pub cashier_user_id: Uuid,
    pub closing_date: NaiveDate,
    pub expected_cash_cents: i64,
    pub actual_cash_cents: i64,
    pub expected_card_cents: i64,
    pub actual_card_cents: i64,
    pub expected_gift_card_cents: i64,
    pub actual_gift_card_cents: i64,
    pub variance_cents: i64,
    pub status: ClosingStatus,
    pub approval_request_id: Option<Uuid>,
    pub notes: Option<String>,
    pub closed_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<RegisterClosing> for RegisterClosingResponse {
    fn from(rc: RegisterClosing) -> Self {
        Self {
            id: rc.id,
            location: rc.location,
            cashier_user_id: rc.cashier_user_id,
            closing_date: rc.closing_date,
            expected_cash_cents: rc.expected_cash_cents,
            actual_cash_cents: rc.actual_cash_cents,
            expected_card_cents: rc.expected_card_cents,
            actual_card_cents: rc.actual_card_cents,
            expected_gift_card_cents: rc.expected_gift_card_cents,
            actual_gift_card_cents: rc.actual_gift_card_cents,
            variance_cents: rc.variance_cents,
            status: rc.status,
            approval_request_id: rc.approval_request_id,
            notes: rc.notes,
            closed_at: rc.closed_at,
            confirmed_at: rc.confirmed_at,
            created_at: rc.created_at,
        }
    }
}

#[derive(Deserialize)]
pub struct CloseRegisterRequest {
    pub location: String,
    pub actual_cash_cents: i64,
    pub actual_card_cents: i64,
    pub actual_gift_card_cents: i64,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct ClosingQueryParams {
    pub location: Option<String>,
    pub date: Option<NaiveDate>,
    pub status: Option<String>,
}
