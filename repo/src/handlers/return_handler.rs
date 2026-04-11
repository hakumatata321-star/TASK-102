use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::approval::{ApprovalRequest, ApprovalStatus, NewApprovalRequest};
use crate::models::ledger_entry::*;
use crate::models::order::*;
use crate::models::order_line_item::*;
use crate::pos::idempotency::{check_idempotency, store_idempotency, reserve_idempotency_key, finalize_idempotency};
use crate::rbac::guard::{check_permission, check_permission_for_request, check_permission_no_approval, resolve_permission_id};
use crate::schema::{approval_requests, ledger_entries, order_line_items, orders};

/// Line item input for the new-items portion of an exchange.
#[derive(Deserialize)]
pub struct ExchangeNewItemInput {
    pub sku: String,
    pub description: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    #[serde(default)]
    pub tax_cents: i64,
}

#[derive(Deserialize)]
pub struct InitiateReturnRequest {
    pub idempotency_key: Uuid,
    pub line_items: Vec<ReturnLineInput>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct ReturnLineInput {
    pub original_line_item_id: Uuid,
    pub quantity: i32, // positive value; will be stored as negative
}

pub async fn initiate_return(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<InitiateReturnRequest>,
) -> Result<HttpResponse, AppError> {
    let source_order_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission_for_request(&auth.0, "order.return", req.method().as_str(), req.path(), &mut conn)?;

    // Atomic idempotency: check for cached replay first
    if let Some(cached) = check_idempotency(body.idempotency_key, &mut conn)? {
        return Ok(cached);
    }

    let source: Order = orders::table
        .find(source_order_id)
        .select(Order::as_select())
        .first(&mut conn)?;

    // Object-level scope enforcement
    ctx.enforce_scope(source.cashier_user_id, source.department.as_deref(), Some(&source.location))?;

    if source.status != OrderStatus::Paid && source.status != OrderStatus::Closed {
        return Err(AppError::Validation(
            "Returns can only be initiated on Paid or Closed orders".into(),
        ));
    }

    let result = conn.transaction::<_, AppError, _>(|conn| {
        // Reserve idempotency key atomically inside transaction
        if let Some(_cached) = reserve_idempotency_key(body.idempotency_key, "return_order", conn)? {
            return Err(AppError::Internal("idempotency_race".into()));
        }
        // Validate return line items against originals
        let mut return_subtotal: i64 = 0;
        let mut return_tax: i64 = 0;
        let mut new_line_items = Vec::new();

        for rli in &body.line_items {
            if rli.quantity <= 0 {
                return Err(AppError::Validation("Return quantity must be positive".into()));
            }

            let original: OrderLineItem = order_line_items::table
                .find(rli.original_line_item_id)
                .select(OrderLineItem::as_select())
                .first(conn)?;

            if original.order_id != source_order_id {
                return Err(AppError::Validation(
                    "Line item does not belong to the source order".into(),
                ));
            }

            if rli.quantity > original.quantity {
                return Err(AppError::Validation(format!(
                    "Return quantity {} exceeds original quantity {} for SKU {}",
                    rli.quantity, original.quantity, original.sku
                )));
            }

            let line_total = -(original.unit_price_cents * rli.quantity as i64);
            let line_tax = -(original.tax_cents * rli.quantity as i64);
            return_subtotal += line_total;
            return_tax += line_tax;

            new_line_items.push(NewOrderLineItem {
                order_id: Uuid::nil(), // placeholder, set after order insert
                sku: original.sku.clone(),
                description: original.description.clone(),
                quantity: -(rli.quantity),
                unit_price_cents: original.unit_price_cents,
                tax_cents: original.tax_cents,
                line_total_cents: line_total,
                original_line_item_id: Some(original.id),
            });
        }

        let return_total = return_subtotal + return_tax;

        // Generate return order number
        let count: i64 = orders::table
            .filter(orders::original_order_id.eq(source_order_id))
            .count()
            .get_result(conn)?;
        let return_number = format!("{}-R{}", source.order_number, count + 1);

        let new_order = NewOrder {
            order_number: return_number,
            status: OrderStatus::ReturnInitiated,
            cashier_user_id: auth.0.sub,
            location: source.location.clone(),
            department: source.department.clone(),
            customer_reference: source.customer_reference.clone(),
            original_order_id: Some(source_order_id),
            subtotal_cents: return_subtotal,
            tax_cents: return_tax,
            total_cents: return_total,
            notes: body.notes.clone(),
        };

        let return_order: Order = diesel::insert_into(orders::table)
            .values(&new_order)
            .get_result(conn)?;

        // Insert return line items
        let mut items = Vec::new();
        for mut nli in new_line_items {
            nli.order_id = return_order.id;
            let item: OrderLineItem = diesel::insert_into(order_line_items::table)
                .values(&nli)
                .get_result(conn)?;
            items.push(item);
        }

        // Transition source order
        diesel::update(orders::table.find(source_order_id))
            .set((
                orders::status.eq(OrderStatus::ReturnInitiated),
                orders::updated_at.eq(Utc::now()),
            ))
            .execute(conn)?;

        Ok(OrderDetailResponse {
            order: OrderResponse::from(return_order),
            line_items: items.into_iter().map(OrderLineItemResponse::from).collect(),
            ledger_entries: vec![],
        })
    })?;

    let json =
        serde_json::to_value(&result).map_err(|e| AppError::Internal(e.to_string()))?;
    finalize_idempotency(body.idempotency_key, result.order.id, 201, &json, &mut conn)?;

    Ok(HttpResponse::Created().json(result))
}

#[derive(Deserialize)]
pub struct InitiateExchangeRequest {
    pub idempotency_key: Uuid,
    pub return_items: Vec<ReturnLineInput>,
    pub new_items: Vec<ExchangeNewItemInput>,
    pub notes: Option<String>,
}

pub async fn initiate_exchange(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<InitiateExchangeRequest>,
) -> Result<HttpResponse, AppError> {
    let source_order_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission_for_request(&auth.0, "order.exchange", req.method().as_str(), req.path(), &mut conn)?;

    // Atomic idempotency: check for cached replay
    if let Some(cached) = check_idempotency(body.idempotency_key, &mut conn)? {
        return Ok(cached);
    }

    let source: Order = orders::table
        .find(source_order_id)
        .select(Order::as_select())
        .first(&mut conn)?;

    ctx.enforce_scope(source.cashier_user_id, source.department.as_deref(), Some(&source.location))?;

    if source.status != OrderStatus::Paid && source.status != OrderStatus::Closed {
        return Err(AppError::Validation(
            "Exchanges can only be initiated on Paid or Closed orders".into(),
        ));
    }

    let result = conn.transaction::<_, AppError, _>(|conn| {
        // Reserve idempotency key atomically inside transaction
        if let Some(_cached) = reserve_idempotency_key(body.idempotency_key, "exchange_order", conn)? {
            return Err(AppError::Internal("idempotency_race".into()));
        }
        // --- Return portion ---
        let mut return_subtotal: i64 = 0;
        let mut return_tax: i64 = 0;
        let mut return_line_items = Vec::new();

        for rli in &body.return_items {
            let original: OrderLineItem = order_line_items::table
                .find(rli.original_line_item_id)
                .select(OrderLineItem::as_select())
                .first(conn)?;

            if original.order_id != source_order_id {
                return Err(AppError::Validation(
                    "Line item does not belong to the source order".into(),
                ));
            }

            let line_total = -(original.unit_price_cents * rli.quantity as i64);
            let line_tax = -(original.tax_cents * rli.quantity as i64);
            return_subtotal += line_total;
            return_tax += line_tax;

            return_line_items.push(NewOrderLineItem {
                order_id: Uuid::nil(),
                sku: original.sku.clone(),
                description: original.description.clone(),
                quantity: -(rli.quantity),
                unit_price_cents: original.unit_price_cents,
                tax_cents: original.tax_cents,
                line_total_cents: line_total,
                original_line_item_id: Some(original.id),
            });
        }

        // --- New items portion ---
        let mut new_subtotal: i64 = 0;
        let mut new_tax: i64 = 0;
        let mut exchange_line_items = Vec::new();

        for li in &body.new_items {
            let line_total = li.unit_price_cents * li.quantity as i64;
            new_subtotal += line_total;
            new_tax += li.tax_cents * li.quantity as i64;

            exchange_line_items.push(NewOrderLineItem {
                order_id: Uuid::nil(),
                sku: li.sku.clone(),
                description: li.description.clone(),
                quantity: li.quantity,
                unit_price_cents: li.unit_price_cents,
                tax_cents: li.tax_cents,
                line_total_cents: line_total,
                original_line_item_id: None,
            });
        }

        // Create exchange order combining return + new items
        let net_subtotal = return_subtotal + new_subtotal;
        let net_tax = return_tax + new_tax;
        let net_total = net_subtotal + net_tax;

        let count: i64 = orders::table
            .filter(orders::original_order_id.eq(source_order_id))
            .count()
            .get_result(conn)?;
        let exchange_number = format!("{}-X{}", source.order_number, count + 1);

        let new_order = NewOrder {
            order_number: exchange_number,
            status: OrderStatus::Draft,
            cashier_user_id: auth.0.sub,
            location: source.location.clone(),
            department: source.department.clone(),
            customer_reference: source.customer_reference.clone(),
            original_order_id: Some(source_order_id),
            subtotal_cents: net_subtotal,
            tax_cents: net_tax,
            total_cents: net_total,
            notes: body.notes.clone(),
        };

        let exchange_order: Order = diesel::insert_into(orders::table)
            .values(&new_order)
            .get_result(conn)?;

        let mut all_items = Vec::new();
        for mut nli in return_line_items {
            nli.order_id = exchange_order.id;
            let item: OrderLineItem = diesel::insert_into(order_line_items::table)
                .values(&nli)
                .get_result(conn)?;
            all_items.push(item);
        }
        for mut nli in exchange_line_items {
            nli.order_id = exchange_order.id;
            let item: OrderLineItem = diesel::insert_into(order_line_items::table)
                .values(&nli)
                .get_result(conn)?;
            all_items.push(item);
        }

        Ok(OrderDetailResponse {
            order: OrderResponse::from(exchange_order),
            line_items: all_items.into_iter().map(OrderLineItemResponse::from).collect(),
            ledger_entries: vec![],
        })
    })?;

    let json =
        serde_json::to_value(&result).map_err(|e| AppError::Internal(e.to_string()))?;
    finalize_idempotency(body.idempotency_key, result.order.id, 201, &json, &mut conn)?;

    Ok(HttpResponse::Created().json(result))
}

#[derive(Deserialize)]
pub struct InitiateReversalRequest {
    pub idempotency_key: Uuid,
    pub notes: Option<String>,
}

/// Initiate a reversal — creates an approval request but does NOT mutate
/// financial state. The actual reversal executes only after approval.
pub async fn initiate_reversal(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<InitiateReversalRequest>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;

    // Atomic idempotency: check for cached replay
    if let Some(cached) = check_idempotency(body.idempotency_key, &mut conn)? {
        return Ok(cached);
    }

    // Object-level scope enforcement on target order
    let scope_ctx = check_permission_no_approval(&auth.0, "order.read", &mut conn)?;

    let order: Order = orders::table
        .find(order_id)
        .select(Order::as_select())
        .first(&mut conn)?;

    scope_ctx.enforce_scope(order.cashier_user_id, order.department.as_deref(), Some(&order.location))?;

    if order.status != OrderStatus::Paid && order.status != OrderStatus::Closed {
        return Err(AppError::Validation(
            "Reversals can only be initiated on Paid or Closed orders".into(),
        ));
    }

    // Determine which permission applies
    let is_late = order.created_at < Utc::now() - Duration::hours(24);
    let perm_code = if is_late { "order.reverse_late" } else { "order.reverse" };
    check_permission_no_approval(&auth.0, perm_code, &mut conn)?;

    let perm_id = resolve_permission_id(perm_code, &mut conn)?;

    // Wrap idempotency reservation + mutations in a transaction so the
    // sentinel key is rolled back if any step fails, allowing client retry.
    let (_approval_id, response) = conn.transaction::<_, AppError, _>(|conn| {
        // Reserve idempotency key atomically inside transaction
        if let Some(_cached) = reserve_idempotency_key(body.idempotency_key, "reversal_request", conn)? {
            return Err(AppError::Conflict("Duplicate reversal request in progress".into()));
        }

        let payload = serde_json::json!({
            "type": "order_reversal",
            "order_id": order_id,
            "order_number": order.order_number,
            "idempotency_key": body.idempotency_key,
            "is_late_reversal": is_late,
            "note": body.notes,
            "requested_by": auth.0.sub,
        });

        let approval: ApprovalRequest = diesel::insert_into(approval_requests::table)
            .values(&NewApprovalRequest {
                permission_point_id: perm_id,
                requester_user_id: auth.0.sub,
                payload,
            })
            .returning(ApprovalRequest::as_returning())
            .get_result(conn)?;

        // Mark order as ReversalPending (no ledger changes)
        diesel::update(orders::table.find(order_id))
            .set((
                orders::status.eq(OrderStatus::ReversalPending),
                orders::notes.eq(body.notes.as_deref()),
                orders::updated_at.eq(Utc::now()),
            ))
            .execute(conn)?;

        let response = serde_json::json!({
            "message": "Reversal requires approval before financial mutation",
            "approval_request_id": approval.id,
            "order_id": order_id,
            "status": "reversal_pending",
        });

        Ok((approval.id, response))
    })?;

    let json = serde_json::to_value(&response).unwrap();
    finalize_idempotency(body.idempotency_key, order_id, 202, &json, &mut conn)?;

    Ok(HttpResponse::Accepted().json(response))
}

#[derive(Deserialize)]
pub struct ExecuteReversalRequest {
    pub approval_request_id: Uuid,
    pub idempotency_key: Uuid,
}

/// Execute a reversal ONLY after the linked approval is Approved.
/// This is where financial mutation (ledger entries) actually happens.
pub async fn execute_reversal(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<ExecuteReversalRequest>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;

    // Atomic idempotency: check for cached replay
    if let Some(cached) = check_idempotency(body.idempotency_key, &mut conn)? {
        return Ok(cached);
    }

    let rev_ctx = check_permission_no_approval(&auth.0, "order.reverse", &mut conn)?;

    // Verify approval is Approved
    let approval: ApprovalRequest = approval_requests::table
        .find(body.approval_request_id)
        .select(ApprovalRequest::as_select())
        .first(&mut conn)
        .map_err(|_| AppError::NotFound("Approval request not found".into()))?;

    if approval.status != ApprovalStatus::Approved {
        return Err(AppError::Validation(
            "Reversal cannot execute: approval not yet approved".into(),
        ));
    }

    // Verify the approval is for this order
    let approval_order_id = approval.payload["order_id"]
        .as_str()
        .and_then(|s| s.parse::<Uuid>().ok());
    if approval_order_id != Some(order_id) {
        return Err(AppError::Validation(
            "Approval request does not match this order".into(),
        ));
    }

    let order: Order = orders::table
        .find(order_id)
        .select(Order::as_select())
        .first(&mut conn)?;

    // Object-level scope enforcement
    rev_ctx.enforce_scope(order.cashier_user_id, order.department.as_deref(), Some(&order.location))?;

    if order.status != OrderStatus::ReversalPending {
        return Err(AppError::Validation(
            "Order must be in ReversalPending status to execute reversal".into(),
        ));
    }

    // Capture before-state for audit
    let before = serde_json::json!({"order_id": order_id, "status": "reversal_pending", "total_cents": order.total_cents});

    // NOW perform the financial mutation inside a transaction
    let reversal_entries = conn.transaction::<_, AppError, _>(|conn| {
        // Reserve idempotency key atomically inside transaction
        if let Some(_cached) = reserve_idempotency_key(body.idempotency_key, "reversal_executed", conn)? {
            return Err(AppError::Internal("idempotency_race".into()));
        }

        let original_entries: Vec<LedgerEntry> = ledger_entries::table
            .filter(ledger_entries::order_id.eq(order_id))
            .filter(ledger_entries::entry_kind.eq(LedgerEntryKind::Payment))
            .select(LedgerEntry::as_select())
            .load(conn)?;

        let mut reversals = Vec::new();
        for entry in &original_entries {
            let reversal = NewLedgerEntry {
                order_id,
                tender_type: entry.tender_type.clone(),
                entry_kind: LedgerEntryKind::Reversal,
                amount_cents: -(entry.amount_cents),
                reference_code: Some(format!("REVERSAL-OF-{}", entry.id)),
                idempotency_key: Uuid::new_v4(),
                created_by: auth.0.sub,
            };
            let created: LedgerEntry = diesel::insert_into(ledger_entries::table)
                .values(&reversal)
                .get_result(conn)?;
            reversals.push(created);
        }

        diesel::update(orders::table.find(order_id))
            .set((
                orders::status.eq(OrderStatus::Reversed),
                orders::updated_at.eq(Utc::now()),
            ))
            .execute(conn)?;

        Ok(reversals)
    })?;

    // Audit the financial mutation with before/after state
    let after = serde_json::json!({"order_id": order_id, "status": "reversed", "reversal_count": reversal_entries.len()});
    let _ = crate::audit::service::audit_write(
        &mut conn, auth.0.sub, "reversal", "orders", Some(order_id), Some(&before), Some(&after),
    );

    let updated: Order = orders::table.find(order_id).select(Order::as_select()).first(&mut conn)?;
    let items: Vec<OrderLineItem> = order_line_items::table
        .filter(order_line_items::order_id.eq(order_id))
        .select(OrderLineItem::as_select())
        .load(&mut conn)?;

    let response = OrderDetailResponse {
        order: OrderResponse::from(updated),
        line_items: items.into_iter().map(OrderLineItemResponse::from).collect(),
        ledger_entries: reversal_entries.into_iter().map(LedgerEntryResponse::from).collect(),
    };

    let json = serde_json::to_value(&response).map_err(|e| AppError::Internal(e.to_string()))?;
    finalize_idempotency(body.idempotency_key, order_id, 200, &json, &mut conn)?;

    Ok(HttpResponse::Ok().json(response))
}
