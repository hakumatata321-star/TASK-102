use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::schema::order_line_items;

#[derive(Debug, Queryable, Identifiable, Selectable, Clone)]
#[diesel(table_name = order_line_items)]
pub struct OrderLineItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub sku: String,
    pub description: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub tax_cents: i64,
    pub line_total_cents: i64,
    pub original_line_item_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = order_line_items)]
pub struct NewOrderLineItem {
    pub order_id: Uuid,
    pub sku: String,
    pub description: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub tax_cents: i64,
    pub line_total_cents: i64,
    pub original_line_item_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct OrderLineItemResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub sku: String,
    pub description: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub tax_cents: i64,
    pub line_total_cents: i64,
    pub original_line_item_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<OrderLineItem> for OrderLineItemResponse {
    fn from(li: OrderLineItem) -> Self {
        Self {
            id: li.id,
            order_id: li.order_id,
            sku: li.sku,
            description: li.description,
            quantity: li.quantity,
            unit_price_cents: li.unit_price_cents,
            tax_cents: li.tax_cents,
            line_total_cents: li.line_total_cents,
            original_line_item_id: li.original_line_item_id,
            created_at: li.created_at,
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateLineItemInput {
    #[validate(length(min = 1, max = 128))]
    pub sku: String,
    #[validate(length(min = 1, max = 512))]
    pub description: String,
    #[validate(range(min = 1))]
    pub quantity: i32,
    #[validate(range(min = 0))]
    pub unit_price_cents: i64,
    #[serde(default)]
    #[validate(range(min = 0))]
    pub tax_cents: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    fn valid_input() -> CreateLineItemInput {
        CreateLineItemInput {
            sku: "SKU-001".to_string(),
            description: "Test item".to_string(),
            quantity: 2,
            unit_price_cents: 1000,
            tax_cents: 80,
        }
    }

    #[test]
    fn test_valid_line_item_passes_validation() {
        let input = valid_input();
        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_negative_unit_price_rejected() {
        let mut input = valid_input();
        input.unit_price_cents = -500;
        let result = input.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("unit_price_cents"));
    }

    #[test]
    fn test_negative_tax_cents_rejected() {
        let mut input = valid_input();
        input.tax_cents = -100;
        let result = input.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("tax_cents"));
    }

    #[test]
    fn test_zero_price_accepted() {
        let mut input = valid_input();
        input.unit_price_cents = 0;
        input.tax_cents = 0;
        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_zero_quantity_rejected() {
        let mut input = valid_input();
        input.quantity = 0;
        let result = input.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("quantity"));
    }
}
