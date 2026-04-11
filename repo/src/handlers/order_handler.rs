use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::ledger_entry::*;
use crate::models::order::*;
use crate::models::order_line_item::*;
use crate::models::receipt::*;
use crate::pos::idempotency::{check_idempotency, store_idempotency, reserve_idempotency_key, finalize_idempotency};
use crate::pos::state_machine;
use crate::rbac::guard::{check_permission, check_permission_for_request, check_permission_no_approval};
use crate::schema::{ledger_entries, order_line_items, orders, receipts};

/// Helper: request-aware permission check using HttpRequest context.
fn check_perm_req(
    auth: &crate::auth::jwt::Claims,
    code: &str,
    req: &HttpRequest,
    conn: &mut diesel::PgConnection,
) -> Result<crate::rbac::data_scope::PermissionContext, AppError> {
    check_permission_for_request(auth, code, req.method().as_str(), req.path(), conn)
}

/// Generate a human-readable order number: LOC-YYYYMMDD-XXXX
fn generate_order_number(location: &str, conn: &mut PgConnection) -> Result<String, AppError> {
    let date = Utc::now().format("%Y%m%d");
    let prefix = format!(
        "{}-{}",
        &location[..std::cmp::min(location.len(), 3)].to_uppercase(),
        date
    );

    // Count existing orders with same prefix today
    let count: i64 = orders::table
        .filter(orders::order_number.like(format!("{}%", prefix)))
        .count()
        .get_result(conn)?;

    Ok(format!("{}-{:04}", prefix, count + 1))
}

pub async fn create_order(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<CreateOrderRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_perm_req(&auth.0, "order.create", &req, &mut conn)?;

    // Compute totals from line items
    let mut subtotal: i64 = 0;
    let mut tax: i64 = 0;
    for li in &body.line_items {
        let line_total = li.unit_price_cents * li.quantity as i64;
        subtotal += line_total;
        tax += li.tax_cents * li.quantity as i64;
    }
    let total = subtotal + tax;

    let order_number = generate_order_number(&body.location, &mut conn)?;

    // Insert order + line items in a transaction
    let (order, items) = conn.transaction::<_, AppError, _>(|conn| {
        let new_order = NewOrder {
            order_number,
            status: OrderStatus::Draft,
            cashier_user_id: auth.0.sub,
            location: body.location.clone(),
            department: body.department.clone(),
            customer_reference: body.customer_reference.clone(),
            original_order_id: None,
            subtotal_cents: subtotal,
            tax_cents: tax,
            total_cents: total,
            notes: body.notes.clone(),
        };

        let order: Order = diesel::insert_into(orders::table)
            .values(&new_order)
            .get_result(conn)?;

        let mut items = Vec::new();
        for li in &body.line_items {
            let line_total = li.unit_price_cents * li.quantity as i64;
            let new_item = NewOrderLineItem {
                order_id: order.id,
                sku: li.sku.clone(),
                description: li.description.clone(),
                quantity: li.quantity,
                unit_price_cents: li.unit_price_cents,
                tax_cents: li.tax_cents,
                line_total_cents: line_total,
                original_line_item_id: None,
            };
            let item: OrderLineItem = diesel::insert_into(order_line_items::table)
                .values(&new_item)
                .get_result(conn)?;
            items.push(item);
        }

        Ok((order, items))
    })?;

    // Audit: create with after-state hash
    let after = serde_json::json!({"order_id": order.id, "order_number": &order.order_number, "total_cents": order.total_cents});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "create", "orders", Some(order.id), None, Some(&after));

    let response = OrderDetailResponse {
        order: OrderResponse::from(order),
        line_items: items.into_iter().map(OrderLineItemResponse::from).collect(),
        ledger_entries: vec![],
    };

    Ok(HttpResponse::Created().json(response))
}

pub async fn list_orders(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    query: web::Query<OrderQueryParams>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm_req(&auth.0, "order.read", &req, &mut conn)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    let mut q = orders::table.into_boxed();

    // Data-scope filtering
    match ctx.data_scope.as_str() {
        "individual" => {
            q = q.filter(orders::cashier_user_id.eq(ctx.user_id));
        }
        "location" => {
            if let Some(ref loc) = ctx.location {
                q = q.filter(orders::location.eq(loc));
            }
        }
        "department" => {
            if let Some(ref dept) = ctx.department {
                q = q.filter(orders::department.eq(dept));
            }
        }
        _ => {} // unrestricted
    }

    // Query param filters
    if let Some(ref status) = query.status {
        q = q.filter(orders::status.eq(status_from_str(status)?));
    }
    if let Some(ref loc) = query.location {
        q = q.filter(orders::location.eq(loc));
    }

    let results: Vec<Order> = q
        .order(orders::created_at.desc())
        .offset(offset)
        .limit(per_page)
        .select(Order::as_select())
        .load(&mut conn)?;

    let responses: Vec<OrderResponse> = results.into_iter().map(OrderResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_order(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm_req(&auth.0, "order.read", &req, &mut conn)?;

    let order: Order = orders::table
        .find(order_id)
        .select(Order::as_select())
        .first(&mut conn)?;

    // Data-scope check
    if !ctx.owner_in_scope(order.cashier_user_id)
        || !ctx.location_in_scope(Some(&order.location))
        || !ctx.department_in_scope(order.department.as_deref())
    {
        return Err(AppError::Forbidden("Out of data scope".into()));
    }

    let items: Vec<OrderLineItem> = order_line_items::table
        .filter(order_line_items::order_id.eq(order_id))
        .select(OrderLineItem::as_select())
        .load(&mut conn)?;

    let entries: Vec<LedgerEntry> = ledger_entries::table
        .filter(ledger_entries::order_id.eq(order_id))
        .select(LedgerEntry::as_select())
        .load(&mut conn)?;

    let response = OrderDetailResponse {
        order: OrderResponse::from(order),
        line_items: items.into_iter().map(OrderLineItemResponse::from).collect(),
        ledger_entries: entries.into_iter().map(LedgerEntryResponse::from).collect(),
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn update_order(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateOrderRequest>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();

    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm_req(&auth.0, "order.update", &req, &mut conn)?;

    let order: Order = orders::table
        .find(order_id)
        .select(Order::as_select())
        .first(&mut conn)?;

    ctx.enforce_scope(order.cashier_user_id, order.department.as_deref(), Some(&order.location))?;

    // Only allow updates in Draft or Open status
    if order.status != OrderStatus::Draft && order.status != OrderStatus::Open {
        return Err(AppError::Validation(
            "Order can only be updated in Draft or Open status".into(),
        ));
    }

    conn.transaction::<_, AppError, _>(|conn| {
        // If line items are provided, replace them
        if let Some(ref new_items) = body.line_items {
            diesel::delete(order_line_items::table.filter(order_line_items::order_id.eq(order_id)))
                .execute(conn)?;

            let mut subtotal: i64 = 0;
            let mut tax: i64 = 0;
            for li in new_items {
                let line_total = li.unit_price_cents * li.quantity as i64;
                subtotal += line_total;
                tax += li.tax_cents * li.quantity as i64;

                let new_item = NewOrderLineItem {
                    order_id,
                    sku: li.sku.clone(),
                    description: li.description.clone(),
                    quantity: li.quantity,
                    unit_price_cents: li.unit_price_cents,
                    tax_cents: li.tax_cents,
                    line_total_cents: line_total,
                    original_line_item_id: None,
                };
                diesel::insert_into(order_line_items::table)
                    .values(&new_item)
                    .execute(conn)?;
            }

            let total = subtotal + tax;
            diesel::update(orders::table.find(order_id))
                .set((
                    orders::subtotal_cents.eq(subtotal),
                    orders::tax_cents.eq(tax),
                    orders::total_cents.eq(total),
                    orders::updated_at.eq(Utc::now()),
                ))
                .execute(conn)?;
        }

        // Update other fields
        let changeset = UpdateOrder {
            status: None,
            customer_reference: body.customer_reference.clone(),
            subtotal_cents: None,
            tax_cents: None,
            total_cents: None,
            notes: body.notes.clone(),
            updated_at: Utc::now(),
        };
        diesel::update(orders::table.find(order_id))
            .set(&changeset)
            .execute(conn)?;

        Ok(())
    })?;

    // Re-fetch and return
    let updated: Order = orders::table
        .find(order_id)
        .select(Order::as_select())
        .first(&mut conn)?;
    let items: Vec<OrderLineItem> = order_line_items::table
        .filter(order_line_items::order_id.eq(order_id))
        .select(OrderLineItem::as_select())
        .load(&mut conn)?;

    let response = OrderDetailResponse {
        order: OrderResponse::from(updated),
        line_items: items.into_iter().map(OrderLineItemResponse::from).collect(),
        ledger_entries: vec![],
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn transition_order(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<TransitionOrderRequest>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;

    let ctx = check_perm_req(&auth.0, "order.transition", &req, &mut conn)?;

    let order: Order = orders::table
        .find(order_id)
        .select(Order::as_select())
        .first(&mut conn)?;

    // Object-level scope enforcement on target order
    ctx.enforce_scope(order.cashier_user_id, order.department.as_deref(), Some(&order.location))?;

    // Validate transition
    if !state_machine::valid_transition(&order.status, &body.target_status) {
        return Err(AppError::Validation(format!(
            "Invalid transition from {:?} to {:?}",
            order.status, body.target_status
        )));
    }

    // Check extra permissions for specific transitions
    if let Some(extra_perm) = state_machine::extra_permission_for_transition(&body.target_status) {
        // For reversals on orders older than 24h, use the late-reversal permission
        if (body.target_status == OrderStatus::ReversalPending
            || body.target_status == OrderStatus::Reversed)
            && order.created_at < Utc::now() - Duration::hours(24)
        {
            check_permission_no_approval(&auth.0, "order.reverse_late", &mut conn)?;
        } else {
            check_permission_no_approval(&auth.0, extra_perm, &mut conn)?;
        }
    }

    // For stock/accounting-impacting transitions, check idempotency
    let is_impacting = matches!(
        body.target_status,
        OrderStatus::Returned | OrderStatus::Reversed
    );
    if is_impacting {
        let idemp_key = body.idempotency_key.ok_or_else(|| {
            AppError::Validation("idempotency_key required for this transition".into())
        })?;
        if let Some(cached) = check_idempotency(idemp_key, &mut conn)? {
            return Ok(cached);
        }
    }

    // Capture before-state for audit
    let before_state = serde_json::json!({"order_id": order_id, "status": format!("{:?}", order.status)});

    // Perform the transition
    diesel::update(orders::table.find(order_id))
        .set((
            orders::status.eq(&body.target_status),
            orders::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)?;

    let updated: Order = orders::table
        .find(order_id)
        .select(Order::as_select())
        .first(&mut conn)?;

    let after_state = serde_json::json!({"order_id": order_id, "status": format!("{:?}", body.target_status)});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "update", "orders", Some(order_id), Some(&before_state), Some(&after_state));

    let response = OrderResponse::from(updated);

    // Store idempotency for impacting transitions
    if is_impacting {
        if let Some(key) = body.idempotency_key {
            let json = serde_json::to_value(&response)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            store_idempotency(key, "order_transition", order_id, 200, &json, &mut conn)?;
        }
    }

    Ok(HttpResponse::Ok().json(response))
}

pub async fn add_payment(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<AddPaymentRequest>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm_req(&auth.0, "order.add_payment", &req, &mut conn)?;

    // Atomic idempotency: reserve key before mutation, inside validation scope
    if let Some(cached) = check_idempotency(body.idempotency_key, &mut conn)? {
        return Ok(cached);
    }

    let order: Order = orders::table
        .find(order_id)
        .select(Order::as_select())
        .first(&mut conn)?;

    ctx.enforce_scope(order.cashier_user_id, order.department.as_deref(), Some(&order.location))?;

    if order.status != OrderStatus::Tendering {
        return Err(AppError::Validation(
            "Payments can only be added when order is in Tendering status".into(),
        ));
    }

    if body.amount_cents <= 0 {
        return Err(AppError::Validation("Payment amount must be positive".into()));
    }

    let entry = conn.transaction::<_, AppError, _>(|conn| {
        // Reserve idempotency key atomically inside transaction
        if let Some(cached_in_tx) = reserve_idempotency_key(body.idempotency_key, "ledger_entry", conn)? {
            // Race: another tx completed first — this is unreachable after outer check,
            // but provides defense-in-depth.
            return Err(AppError::Internal("idempotency_race".into()));
        }

        let new_entry = NewLedgerEntry {
            order_id,
            tender_type: body.tender_type.clone(),
            entry_kind: LedgerEntryKind::Payment,
            amount_cents: body.amount_cents,
            reference_code: body.reference_code.clone(),
            idempotency_key: body.idempotency_key,
            created_by: auth.0.sub,
        };

        let entry: LedgerEntry = diesel::insert_into(ledger_entries::table)
            .values(&new_entry)
            .get_result(conn)?;

        // Check if total payments meet or exceed order total
        let paid_amounts: Vec<i64> = ledger_entries::table
            .filter(ledger_entries::order_id.eq(order_id))
            .filter(ledger_entries::entry_kind.eq(LedgerEntryKind::Payment))
            .select(ledger_entries::amount_cents)
            .load(conn)?;
        let total_paid: i64 = paid_amounts.iter().sum();

        if total_paid >= order.total_cents {
            diesel::update(orders::table.find(order_id))
                .set((
                    orders::status.eq(OrderStatus::Paid),
                    orders::updated_at.eq(Utc::now()),
                ))
                .execute(conn)?;
        }

        Ok(entry)
    })?;

    let response = LedgerEntryResponse::from(entry);

    let after = serde_json::json!({"order_id": order_id, "entry_id": response.id, "amount_cents": response.amount_cents});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "create", "ledger_entries", Some(response.id), None, Some(&after));

    let json =
        serde_json::to_value(&response).map_err(|e| AppError::Internal(e.to_string()))?;
    // Finalize the atomically reserved key with actual response
    finalize_idempotency(body.idempotency_key, response.id, 201, &json, &mut conn)?;

    Ok(HttpResponse::Created().json(response))
}

pub async fn list_payments(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let order_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm_req(&auth.0, "order.read", &req, &mut conn)?;

    let order: Order = orders::table.find(order_id).select(Order::as_select()).first(&mut conn)?;
    ctx.enforce_scope(order.cashier_user_id, order.department.as_deref(), Some(&order.location))?;

    let entries: Vec<LedgerEntry> = ledger_entries::table
        .filter(ledger_entries::order_id.eq(order_id))
        .select(LedgerEntry::as_select())
        .order(ledger_entries::created_at.asc())
        .load(&mut conn)?;

    let responses: Vec<LedgerEntryResponse> =
        entries.into_iter().map(LedgerEntryResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

/// Attach a receipt to an order. Accepts multipart/form-data with a file upload.
/// The file is stored locally with SHA-256 fingerprint. Allowed types: PDF/JPG/PNG/CSV/XLSX.
/// Max 10 MB. Also accepts a JSON `receipt_data` field for structured metadata.
pub async fn attach_receipt(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    mut payload: actix_multipart::Multipart,
) -> Result<HttpResponse, AppError> {
    use futures_util::StreamExt;

    let order_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_perm_req(&auth.0, "order.attach_receipt", &req, &mut conn)?;

    let order: Order = orders::table
        .find(order_id)
        .select(Order::as_select())
        .first(&mut conn)?;

    ctx.enforce_scope(order.cashier_user_id, order.department.as_deref(), Some(&order.location))?;

    let count: i64 = receipts::table
        .filter(receipts::order_id.eq(order_id))
        .count()
        .get_result(&mut conn)?;
    let receipt_number = format!("RCP-{}-{}", order_id.to_string()[..8].to_uppercase(), count + 1);

    let mut file_path_stored: Option<String> = None;
    let mut content_type_stored: Option<String> = None;
    let mut file_size: Option<i64> = None;
    let mut sha256: Option<String> = None;
    let mut receipt_data = serde_json::json!({});

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::Validation(format!("Multipart error: {}", e)))?;
        let disposition = field.content_disposition()
            .ok_or_else(|| AppError::Validation("Missing content disposition".into()))?
            .clone();
        let field_name = disposition.get_name().unwrap_or("").to_string();

        if field_name == "receipt_data" {
            // JSON metadata field
            let mut json_bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk.map_err(|e| AppError::Internal(e.to_string()))?;
                json_bytes.extend_from_slice(&chunk);
            }
            receipt_data = serde_json::from_slice(&json_bytes)
                .unwrap_or(serde_json::json!({}));
        } else if field_name == "file" {
            // File upload field
            let filename = disposition.get_filename()
                .ok_or_else(|| AppError::Validation("Missing filename".into()))?
                .to_string();

            let ct = crate::storage::content_type_from_filename(&filename)?;
            crate::storage::validate_content_type(&ct)?;

            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk.map_err(|e| AppError::Internal(e.to_string()))?;
                data.extend_from_slice(&chunk);
                if data.len() as u64 > crate::models::file_attachment::MAX_FILE_SIZE {
                    return Err(AppError::Validation("File exceeds 10 MB limit".into()));
                }
            }
            crate::storage::validate_file_size(data.len() as u64)?;

            let (path_on_disk, hash) = crate::storage::save_artifact(
                "receipts", order_id, &filename.split('.').last().unwrap_or("bin"), &data,
            )?;

            // Check duplicate hash for this order
            let dup: Option<Receipt> = receipts::table
                .filter(receipts::order_id.eq(order_id))
                .filter(receipts::sha256_hash.eq(&hash))
                .select(Receipt::as_select())
                .first(&mut conn)
                .optional()?;
            if let Some(d) = dup {
                crate::storage::delete_file(&path_on_disk)?;
                return Err(AppError::Conflict(format!("Duplicate receipt file (hash match with {})", d.id)));
            }

            file_path_stored = Some(path_on_disk);
            content_type_stored = Some(ct);
            file_size = Some(data.len() as i64);
            sha256 = Some(hash);
        }
    }

    let new_receipt = NewReceipt {
        order_id,
        receipt_number,
        receipt_data,
        created_by: auth.0.sub,
        file_path: file_path_stored,
        content_type: content_type_stored,
        file_size_bytes: file_size,
        sha256_hash: sha256,
    };

    let receipt: Receipt = diesel::insert_into(receipts::table)
        .values(&new_receipt)
        .returning(Receipt::as_returning())
        .get_result(&mut conn)?;

    let after = serde_json::json!({"id": receipt.id, "receipt_number": &receipt.receipt_number, "sha256": &receipt.sha256_hash});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "create", "receipts", Some(receipt.id), None, Some(&after));

    Ok(HttpResponse::Created().json(ReceiptResponse::from(receipt)))
}

/// Parse a string into an OrderStatus for query filtering.
fn status_from_str(s: &str) -> Result<OrderStatus, AppError> {
    match s {
        "draft" => Ok(OrderStatus::Draft),
        "open" => Ok(OrderStatus::Open),
        "tendering" => Ok(OrderStatus::Tendering),
        "paid" => Ok(OrderStatus::Paid),
        "closed" => Ok(OrderStatus::Closed),
        "return_initiated" => Ok(OrderStatus::ReturnInitiated),
        "returned" => Ok(OrderStatus::Returned),
        "reversal_pending" => Ok(OrderStatus::ReversalPending),
        "reversed" => Ok(OrderStatus::Reversed),
        _ => Err(AppError::Validation(format!("Invalid order status: {}", s))),
    }
}
