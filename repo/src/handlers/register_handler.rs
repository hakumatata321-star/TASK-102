use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{NaiveTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::approval::{ApprovalRequest, ApprovalStatus, NewApprovalRequest};
use crate::models::ledger_entry::{LedgerEntry, LedgerEntryKind, TenderType};
use crate::models::register_closing::*;
use crate::rbac::guard::{check_permission_for_request, check_permission_no_approval};
use crate::schema::{approval_requests, ledger_entries, orders, register_closings};

/// $20.00 variance threshold in cents
const VARIANCE_THRESHOLD_CENTS: i64 = 2000;

pub async fn close_register(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    body: web::Json<CloseRegisterRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission_for_request(&auth.0, "register.close", req.method().as_str(), req.path(), &mut conn)?;

    let today = Utc::now().date_naive();

    // Compute expected totals from ledger entries for this cashier+location today
    let day_start = today
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .and_utc();
    let day_end = (today + chrono::Duration::days(1))
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .and_utc();

    // Get order IDs for this cashier and location today
    let order_ids: Vec<Uuid> = orders::table
        .filter(orders::cashier_user_id.eq(auth.0.sub))
        .filter(orders::location.eq(&body.location))
        .filter(orders::created_at.ge(day_start))
        .filter(orders::created_at.lt(day_end))
        .select(orders::id)
        .load(&mut conn)?;

    // Load all ledger entries for the day and compute sums in Rust
    let all_entries: Vec<LedgerEntry> = if order_ids.is_empty() {
        vec![]
    } else {
        ledger_entries::table
            .filter(ledger_entries::order_id.eq_any(&order_ids))
            .select(LedgerEntry::as_select())
            .load(&mut conn)?
    };

    let sum_by = |tender: &TenderType| -> i64 {
        all_entries
            .iter()
            .filter(|e| &e.tender_type == tender)
            .map(|e| e.amount_cents) // payments positive, refunds/reversals negative
            .sum()
    };

    let net_expected_cash = sum_by(&TenderType::Cash);
    let net_expected_card = sum_by(&TenderType::Card);
    let net_expected_gift = sum_by(&TenderType::GiftCard);

    let actual_total = body.actual_cash_cents + body.actual_card_cents + body.actual_gift_card_cents;
    let expected_total = net_expected_cash + net_expected_card + net_expected_gift;
    let variance = actual_total - expected_total;

    let needs_approval = variance.abs() > VARIANCE_THRESHOLD_CENTS;

    let (status, approval_id) = if needs_approval {
        // Create an approval request
        let perm_id = crate::rbac::guard::resolve_permission_id(
            "register.confirm_variance",
            &mut conn,
        )?;
        let payload = serde_json::json!({
            "type": "register_variance",
            "location": body.location,
            "cashier_user_id": auth.0.sub,
            "closing_date": today.to_string(),
            "variance_cents": variance,
        });
        let new_approval = NewApprovalRequest {
            permission_point_id: perm_id,
            requester_user_id: auth.0.sub,
            payload,
        };
        let approval: ApprovalRequest = diesel::insert_into(approval_requests::table)
            .values(&new_approval)
            .returning(ApprovalRequest::as_returning())
            .get_result(&mut conn)?;

        (ClosingStatus::VarianceFlagged, Some(approval.id))
    } else {
        (ClosingStatus::Confirmed, None)
    };

    let new_closing = NewRegisterClosing {
        location: body.location.clone(),
        cashier_user_id: auth.0.sub,
        closing_date: today,
        expected_cash_cents: net_expected_cash,
        actual_cash_cents: body.actual_cash_cents,
        expected_card_cents: net_expected_card,
        actual_card_cents: body.actual_card_cents,
        expected_gift_card_cents: net_expected_gift,
        actual_gift_card_cents: body.actual_gift_card_cents,
        variance_cents: variance,
        status,
        approval_request_id: approval_id,
        notes: body.notes.clone(),
    };

    let closing: RegisterClosing = diesel::insert_into(register_closings::table)
        .values(&new_closing)
        .returning(RegisterClosing::as_returning())
        .get_result(&mut conn)?;

    if needs_approval {
        Ok(HttpResponse::Accepted().json(RegisterClosingResponse::from(closing)))
    } else {
        Ok(HttpResponse::Created().json(RegisterClosingResponse::from(closing)))
    }
}

pub async fn list_closings(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    query: web::Query<ClosingQueryParams>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission_for_request(&auth.0, "register.read", req.method().as_str(), req.path(), &mut conn)?;

    let mut q = register_closings::table.into_boxed();

    // Data-scope
    match ctx.data_scope.as_str() {
        "individual" => {
            q = q.filter(register_closings::cashier_user_id.eq(ctx.user_id));
        }
        "location" => {
            if let Some(ref loc) = ctx.location {
                q = q.filter(register_closings::location.eq(loc));
            }
        }
        _ => {}
    }

    if let Some(ref loc) = query.location {
        q = q.filter(register_closings::location.eq(loc));
    }
    if let Some(date) = query.date {
        q = q.filter(register_closings::closing_date.eq(date));
    }
    if let Some(ref status) = query.status {
        q = q.filter(register_closings::status.eq(closing_status_from_str(status)?));
    }

    let results: Vec<RegisterClosing> = q
        .select(RegisterClosing::as_select())
        .order(register_closings::created_at.desc())
        .load(&mut conn)?;

    let responses: Vec<RegisterClosingResponse> =
        results.into_iter().map(RegisterClosingResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_closing(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let closing_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let ctx = check_permission_for_request(&auth.0, "register.read", req.method().as_str(), req.path(), &mut conn)?;

    let closing: RegisterClosing = register_closings::table
        .find(closing_id)
        .select(RegisterClosing::as_select())
        .first(&mut conn)?;

    ctx.enforce_scope(closing.cashier_user_id, None, Some(&closing.location))?;

    Ok(HttpResponse::Ok().json(RegisterClosingResponse::from(closing)))
}

pub async fn confirm_closing(
    pool: web::Data<DbPool>,
    auth: AuthenticatedUser,
    _req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let closing_id = path.into_inner();
    let mut conn = pool.get().map_err(crate::errors::pool_err)?;
    let _ctx = check_permission_no_approval(&auth.0, "register.confirm_variance", &mut conn)?;

    let closing: RegisterClosing = register_closings::table
        .find(closing_id)
        .select(RegisterClosing::as_select())
        .first(&mut conn)?;

    if closing.status != ClosingStatus::VarianceFlagged {
        return Err(AppError::Validation(
            "Only variance-flagged closings can be confirmed".into(),
        ));
    }

    // Check that the linked approval request is approved
    if let Some(approval_id) = closing.approval_request_id {
        let approval: ApprovalRequest = approval_requests::table
            .find(approval_id)
            .select(ApprovalRequest::as_select())
            .first(&mut conn)?;

        if approval.status != ApprovalStatus::Approved {
            return Err(AppError::Validation(
                "Approval request must be approved before confirming".into(),
            ));
        }
    } else {
        return Err(AppError::Validation(
            "No approval request linked to this closing".into(),
        ));
    }

    let before = serde_json::json!({"closing_id": closing_id, "status": "variance_flagged", "variance_cents": closing.variance_cents});

    diesel::update(register_closings::table.find(closing_id))
        .set((
            register_closings::status.eq(ClosingStatus::ManagerConfirmed),
            register_closings::confirmed_at.eq(Some(Utc::now())),
        ))
        .execute(&mut conn)?;

    let updated: RegisterClosing = register_closings::table
        .find(closing_id)
        .select(RegisterClosing::as_select())
        .first(&mut conn)?;

    let after = serde_json::json!({"closing_id": closing_id, "status": "manager_confirmed"});
    let _ = crate::audit::service::audit_write(&mut conn, auth.0.sub, "update", "register_closings", Some(closing_id), Some(&before), Some(&after));

    Ok(HttpResponse::Ok().json(RegisterClosingResponse::from(updated)))
}

fn closing_status_from_str(s: &str) -> Result<ClosingStatus, AppError> {
    match s {
        "pending" => Ok(ClosingStatus::Pending),
        "confirmed" => Ok(ClosingStatus::Confirmed),
        "variance_flagged" => Ok(ClosingStatus::VarianceFlagged),
        "manager_confirmed" => Ok(ClosingStatus::ManagerConfirmed),
        _ => Err(AppError::Validation(format!("Invalid closing status: {}", s))),
    }
}
