use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::orders;

#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[ExistingTypePath = "crate::schema::sql_types::OrderStatusType"]
pub enum OrderStatus {
    #[db_rename = "draft"]
    Draft,
    #[db_rename = "open"]
    Open,
    #[db_rename = "tendering"]
    Tendering,
    #[db_rename = "paid"]
    Paid,
    #[db_rename = "closed"]
    Closed,
    #[db_rename = "return_initiated"]
    ReturnInitiated,
    #[db_rename = "returned"]
    Returned,
    #[db_rename = "reversal_pending"]
    ReversalPending,
    #[db_rename = "reversed"]
    Reversed,
}

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: Uuid,
    pub order_number: String,
    pub status: OrderStatus,
    pub cashier_user_id: Uuid,
    pub location: String,
    pub department: Option<String>,
    pub customer_reference: Option<String>,
    pub original_order_id: Option<Uuid>,
    pub subtotal_cents: i64,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub order_number: String,
    pub status: OrderStatus,
    pub cashier_user_id: Uuid,
    pub location: String,
    pub department: Option<String>,
    pub customer_reference: Option<String>,
    pub original_order_id: Option<Uuid>,
    pub subtotal_cents: i64,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub notes: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = orders)]
pub struct UpdateOrder {
    pub status: Option<OrderStatus>,
    pub customer_reference: Option<Option<String>>,
    pub subtotal_cents: Option<i64>,
    pub tax_cents: Option<i64>,
    pub total_cents: Option<i64>,
    pub notes: Option<Option<String>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub order_number: String,
    pub status: OrderStatus,
    pub cashier_user_id: Uuid,
    pub location: String,
    pub department: Option<String>,
    pub customer_reference: Option<String>,
    pub original_order_id: Option<Uuid>,
    pub subtotal_cents: i64,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Order> for OrderResponse {
    fn from(o: Order) -> Self {
        Self {
            id: o.id,
            order_number: o.order_number,
            status: o.status,
            cashier_user_id: o.cashier_user_id,
            location: o.location,
            department: o.department,
            customer_reference: o.customer_reference,
            original_order_id: o.original_order_id,
            subtotal_cents: o.subtotal_cents,
            tax_cents: o.tax_cents,
            total_cents: o.total_cents,
            notes: o.notes,
            created_at: o.created_at,
            updated_at: o.updated_at,
        }
    }
}

#[derive(Serialize)]
pub struct OrderDetailResponse {
    #[serde(flatten)]
    pub order: OrderResponse,
    pub line_items: Vec<super::order_line_item::OrderLineItemResponse>,
    pub ledger_entries: Vec<super::ledger_entry::LedgerEntryResponse>,
}

#[derive(Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(min = 1, max = 128))]
    pub location: String,
    pub department: Option<String>,
    pub customer_reference: Option<String>,
    pub notes: Option<String>,
    #[validate(nested)]
    pub line_items: Vec<super::order_line_item::CreateLineItemInput>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateOrderRequest {
    pub customer_reference: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    #[validate(nested)]
    pub line_items: Option<Vec<super::order_line_item::CreateLineItemInput>>,
}

#[derive(Deserialize)]
pub struct TransitionOrderRequest {
    pub target_status: OrderStatus,
    pub idempotency_key: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct OrderQueryParams {
    pub status: Option<String>,
    pub location: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
