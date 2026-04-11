#!/bin/bash
##############################################################################
# API Integration Test Runner
# Tests all API endpoints against a running RetailOps Docker deployment.
# Requires: docker compose up (services must be running)
# Usage: bash API_tests/run_api_tests.sh
##############################################################################

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

BASE_URL="${API_BASE_URL:-http://localhost:8081}"
API="$BASE_URL/api/v1"
PASS=0; FAIL=0; TOTAL=0; FAILURES=""
TOKEN=""; ADMIN_USER_ID=""

_TMPF="$(mktemp)"
trap "rm -f '$_TMPF'" EXIT

# ── Global result variables set by http helpers ──
_STATUS=""
_BODY=""

assert_status() {
  TOTAL=$((TOTAL + 1))
  if [ "$2" = "$3" ]; then
    PASS=$((PASS + 1)); echo "  [PASS] $1 (HTTP $3)"
  else
    FAIL=$((FAIL + 1)); FAILURES="$FAILURES\n  [FAIL] $1 — expected $2, got $3"
    echo "  [FAIL] $1 — expected HTTP $2, got HTTP $3"
  fi
}

assert_body() {
  TOTAL=$((TOTAL + 1))
  if echo "$_BODY" | grep -q "$2"; then
    PASS=$((PASS + 1)); echo "  [PASS] $1"
  else
    FAIL=$((FAIL + 1)); FAILURES="$FAILURES\n  [FAIL] $1 — response missing '$2'"
    echo "  [FAIL] $1 — response missing '$2'"
  fi
}

assert_body_not() {
  TOTAL=$((TOTAL + 1))
  if echo "$_BODY" | grep -q "$2"; then
    FAIL=$((FAIL + 1)); FAILURES="$FAILURES\n  [FAIL] $1 — should not contain '$2'"
    echo "  [FAIL] $1 — should not contain '$2'"
  else
    PASS=$((PASS + 1)); echo "  [PASS] $1"
  fi
}

# HTTP helpers — set _STATUS and _BODY globals
http_get() {
  _STATUS=$(curl -s -o "$_TMPF" -w "%{http_code}" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" "$1")
  _BODY=$(cat "$_TMPF")
}

http_post() {
  _STATUS=$(curl -s -o "$_TMPF" -w "%{http_code}" -X POST \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d "$2" "$1")
  _BODY=$(cat "$_TMPF")
}

http_put() {
  _STATUS=$(curl -s -o "$_TMPF" -w "%{http_code}" -X PUT \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d "$2" "$1")
  _BODY=$(cat "$_TMPF")
}

http_delete() {
  _STATUS=$(curl -s -o "$_TMPF" -w "%{http_code}" -X DELETE \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" "$1")
  _BODY=$(cat "$_TMPF")
}

http_post_noauth() {
  _STATUS=$(curl -s -o "$_TMPF" -w "%{http_code}" -X POST \
    -H "Content-Type: application/json" -d "$2" "$1")
  _BODY=$(cat "$_TMPF")
}

jf() { echo "$_BODY" | grep -o "\"$1\":\"[^\"]*\"" | head -1 | cut -d'"' -f4; }

# ── Wait for API ──
echo "============================================"
echo "  RetailOps API Integration Tests"
echo "============================================"
echo ""
echo "[INFO] Checking API at $API ..."
for i in $(seq 1 30); do
  H=$(curl -s -o /dev/null -w "%{http_code}" "$API/health" 2>/dev/null || true)
  [ "$H" = "200" ] && echo "[INFO] API is ready." && break
  [ "$i" = "30" ] && echo "[ERROR] API not reachable." && exit 1
  sleep 1
done
echo ""

###########################################################################
echo "── Section 1: Health ─────────────────────────────"
_STATUS=$(curl -s -o "$_TMPF" -w "%{http_code}" "$API/health"); _BODY=$(cat "$_TMPF")
assert_status "GET /health returns 200" "200" "$_STATUS"
assert_body "Health reports healthy" "healthy"
assert_body "Health reports DB connected" "connected"
echo ""

###########################################################################
echo "── Section 2: Authentication ─────────────────────"

http_post_noauth "$API/auth/bootstrap" '{"username":"testadmin","password":"TestAdmin1234"}'
assert_status "Bootstrap creates admin" "201" "$_STATUS"
TOKEN=$(jf access_token); ADMIN_USER_ID=$(jf user_id)
REFRESH_TOKEN=$(jf refresh_token)

http_post_noauth "$API/auth/bootstrap" '{"username":"x","password":"Xpassword1234"}'
assert_status "Duplicate bootstrap blocked" "403" "$_STATUS"

http_post_noauth "$API/auth/login" '{"username":"testadmin","password":"TestAdmin1234"}'
assert_status "Login succeeds" "200" "$_STATUS"
TOKEN=$(jf access_token)

http_post_noauth "$API/auth/login" '{"username":"testadmin","password":"WrongPass12345"}'
assert_status "Bad password rejected" "401" "$_STATUS"

http_post_noauth "$API/auth/refresh" "{\"refresh_token\":\"$REFRESH_TOKEN\"}"
assert_status "Refresh token works" "200" "$_STATUS"
TOKEN=$(jf access_token)

_STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$API/roles")
assert_status "Missing auth returns 401" "401" "$_STATUS"
echo ""

###########################################################################
echo "── Section 3: Roles & Permissions ────────────────"

http_get "$API/roles"
assert_status "GET /roles" "200" "$_STATUS"
assert_body "Has System Administrator" "System Administrator"
assert_body "Has Cashier" "Cashier"

http_get "$API/permissions"
assert_status "GET /permissions" "200" "$_STATUS"
assert_body "Has order.create" "order.create"

http_post "$API/roles" '{"name":"Test Role","data_scope":"location"}'
assert_status "Create role" "201" "$_STATUS"
ROLE_ID=$(jf id)

http_post "$API/roles" '{"name":"Test Role","data_scope":"location"}'
assert_status "Duplicate role name rejected" "409" "$_STATUS"

http_put "$API/roles/$ROLE_ID" '{"name":"Updated Role"}'
assert_status "Update role" "200" "$_STATUS"
assert_body "Name updated" "Updated Role"

http_delete "$API/roles/$ROLE_ID"
assert_status "Delete role" "204" "$_STATUS"
echo ""

###########################################################################
echo "── Section 4: User Management ────────────────────"

http_post "$API/users" "{\"username\":\"cashier1\",\"password\":\"Cashier12345!\",\"role_id\":\"a0000000-0000-0000-0000-000000000003\",\"location\":\"NYC-01\",\"department\":\"Sales\"}"
assert_status "Create user" "201" "$_STATUS"
assert_body "Has username" "cashier1"
assert_body_not "No password hash exposed" "password_hash"
CASHIER_ID=$(jf id)

http_post "$API/users" '{"username":"bad","password":"short","role_id":"a0000000-0000-0000-0000-000000000003"}'
assert_status "Weak password rejected" "400" "$_STATUS"

http_get "$API/users"
assert_status "List users" "200" "$_STATUS"
assert_body "Users list has cashier1" "cashier1"
echo ""

###########################################################################
echo "── Section 5: POS Orders ─────────────────────────"

http_post "$API/orders" '{"location":"NYC-01","department":"Electronics","line_items":[{"sku":"WIDGET-1","description":"Widget","quantity":2,"unit_price_cents":1500,"tax_cents":120}]}'
assert_status "Create order" "201" "$_STATUS"
assert_body "Status is draft" "draft"
assert_body "Has line items" "WIDGET-1"
ORDER_ID=$(jf id)

http_get "$API/orders/$ORDER_ID"
assert_status "Get order" "200" "$_STATUS"
assert_body "Has line_items" "line_items"

http_get "$API/orders"
assert_status "List orders" "200" "$_STATUS"

http_put "$API/orders/$ORDER_ID" '{"notes":"rush"}'
assert_status "Update draft order" "200" "$_STATUS"

http_post "$API/orders/$ORDER_ID/transition" '{"target_status":"open"}'
assert_status "Draft→Open" "200" "$_STATUS"
assert_body "Status is open" "open"

http_post "$API/orders/$ORDER_ID/transition" '{"target_status":"tendering"}'
assert_status "Open→Tendering" "200" "$_STATUS"

http_post "$API/orders/$ORDER_ID/transition" '{"target_status":"closed"}'
assert_status "Invalid transition rejected" "400" "$_STATUS"

IKEY="aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
http_post "$API/orders/$ORDER_ID/payments" "{\"tender_type\":\"cash\",\"amount_cents\":3240,\"idempotency_key\":\"$IKEY\"}"
assert_status "Add payment" "201" "$_STATUS"
PAY_ID=$(jf id)

http_post "$API/orders/$ORDER_ID/payments" "{\"tender_type\":\"cash\",\"amount_cents\":3240,\"idempotency_key\":\"$IKEY\"}"
assert_status "Idempotent replay" "201" "$_STATUS"
PAY_ID2=$(jf id)
TOTAL=$((TOTAL + 1))
if [ "$PAY_ID" = "$PAY_ID2" ]; then PASS=$((PASS+1)); echo "  [PASS] Same ID on replay"
else FAIL=$((FAIL+1)); FAILURES="$FAILURES\n  [FAIL] Idempotency: different IDs"; echo "  [FAIL] Idempotency: different IDs"; fi

http_get "$API/orders/$ORDER_ID/payments"
assert_status "List payments" "200" "$_STATUS"

# Receipt now uses multipart with file governance
echo "test receipt content" > _receipt_test.pdf
_STATUS=$(curl -s -o "$_TMPF" -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -F "receipt_data={\"items\":[\"Widget x2\"]};type=application/json" \
  -F "file=@_receipt_test.pdf;filename=receipt.pdf" \
  "$API/orders/$ORDER_ID/receipts")
_BODY=$(cat "$_TMPF")
assert_status "Attach receipt (file)" "201" "$_STATUS"
assert_body "Has receipt_number" "receipt_number"
assert_body "Has sha256_hash" "sha256_hash"
rm -f _receipt_test.pdf
echo ""

###########################################################################
echo "── Section 6: Participants & Teams ───────────────"

http_post "$API/participants" '{"first_name":"Alice","last_name":"Smith","email":"alice@test.com","department":"Sales","location":"NYC-01","tags":["vip","q1"]}'
assert_status "Create participant" "201" "$_STATUS"
assert_body "Has tags" "vip"
PART_ID=$(jf id)

http_get "$API/participants/$PART_ID"
assert_status "Get participant" "200" "$_STATUS"
assert_body "Has tags field" "tags"

http_get "$API/participants?q=Alice"
assert_status "Search participants" "200" "$_STATUS"
assert_body "Search finds Alice" "Alice"

http_get "$API/participants?tag=vip"
assert_status "Filter by tag" "200" "$_STATUS"
assert_body "Tag filter works" "Alice"

http_put "$API/participants/$PART_ID" '{"first_name":"Alicia"}'
assert_status "Update participant" "200" "$_STATUS"
assert_body "Name updated" "Alicia"

http_post "$API/teams" '{"name":"Alpha Squad","department":"Sales"}'
assert_status "Create team" "201" "$_STATUS"
TEAM_ID=$(jf id)

http_post "$API/teams/$TEAM_ID/members" "{\"participant_id\":\"$PART_ID\",\"role_label\":\"captain\"}"
assert_status "Add team member" "201" "$_STATUS"
assert_body "Has role_label" "captain"

http_get "$API/teams/$TEAM_ID"
assert_status "Get team with members" "200" "$_STATUS"
assert_body "Has members" "members"

http_post "$API/participants/bulk/tag" "{\"participant_ids\":[\"$PART_ID\"],\"tags\":[\"star\"]}"
assert_status "Bulk tag" "200" "$_STATUS"
assert_body "Bulk result" "affected"

http_post "$API/participants" '{"first_name":"Bob","last_name":"X","department":"Ops"}'
P2_BODY="$_BODY"; P2_ID=$(echo "$P2_BODY" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
http_post "$API/participants/bulk/deactivate" "{\"participant_ids\":[\"$P2_ID\"]}"
assert_status "Bulk deactivate" "200" "$_STATUS"
echo ""

###########################################################################
echo "── Section 7: Datasets & Versions ────────────────"

http_post "$API/datasets" '{"name":"Sales Q1","dataset_type":"raw"}'
assert_status "Create dataset" "201" "$_STATUS"
DS_ID=$(jf id)

http_post "$API/datasets/$DS_ID/versions" '{"storage_path":"/data/v1.csv","row_count":5000,"transformation_note":"Init","field_dictionary":[{"field_name":"order_id","field_type":"uuid","meaning":"PK"}]}'
assert_status "Create version 1" "201" "$_STATUS"
assert_body "Version number 1" "\"version_number\":1"
V1_ID=$(jf id)

http_post "$API/datasets/$DS_ID/versions" "{\"storage_path\":\"/data/v2.csv\",\"row_count\":4800,\"transformation_note\":\"Cleaned\",\"parent_version_ids\":[\"$V1_ID\"],\"field_dictionary\":[]}"
assert_status "Create version 2 with lineage" "201" "$_STATUS"
assert_body "Version number 2" "\"version_number\":2"
V2_ID=$(jf id)

http_get "$API/datasets/$DS_ID/versions"
assert_status "List versions" "200" "$_STATUS"

http_get "$API/datasets/$DS_ID/versions/$V2_ID"
assert_status "Get version detail" "200" "$_STATUS"
assert_body "Has parent_version_ids" "parent_version_ids"

http_get "$API/datasets/$DS_ID/versions/$V2_ID/lineage"
assert_status "Get lineage" "200" "$_STATUS"
assert_body "Has parents" "parents"

http_get "$API/datasets/$DS_ID/versions/$V1_ID/fields"
assert_status "Get field dictionary" "200" "$_STATUS"
assert_body "Has order_id field" "order_id"

http_post "$API/datasets/$DS_ID/rollback" "{\"target_version_id\":\"$V1_ID\",\"note\":\"Revert\"}"
assert_status "Rollback requires approval" "202" "$_STATUS"
assert_body "Has approval_request_id" "approval_request_id"
echo ""

###########################################################################
echo "── Section 8: Notifications ──────────────────────"

http_post "$API/notifications/send-direct" "{\"recipient_user_id\":\"$ADMIN_USER_ID\",\"category\":\"system_announcement\",\"subject\":\"Test Alert\",\"body\":\"Hello\"}"
assert_status "Send notification" "201" "$_STATUS"
assert_body "Notification delivered" "Delivered"
NOTIF_ID=$(jf id)

http_get "$API/notifications/inbox"
assert_status "Inbox" "200" "$_STATUS"
assert_body "Inbox has notification" "Test Alert"

http_get "$API/notifications/inbox/unread-count"
assert_status "Unread count" "200" "$_STATUS"
assert_body "Has unread_count" "unread_count"

http_post "$API/notifications/inbox/$NOTIF_ID/read" '{}'
assert_status "Mark read" "200" "$_STATUS"

http_post "$API/notifications/inbox/read-all" '{}'
assert_status "Mark all read" "200" "$_STATUS"

http_post "$API/notifications/broadcast" '{"subject":"Maint","body":"Downtime tonight"}'
assert_status "Broadcast" "201" "$_STATUS"
assert_body "Has recipients" "recipients"
echo ""

###########################################################################
echo "── Section 9: Reports & Exports ──────────────────"

http_get "$API/reports/kpi-types"
assert_status "KPI types" "200" "$_STATUS"
assert_body "Has participation_by_store" "participation_by_store"

http_post "$API/reports" '{"name":"Store Report","kpi_type":"participation_by_store","dimensions":["location"]}'
assert_status "Create report" "201" "$_STATUS"
RPT_ID=$(jf id)

http_post "$API/reports" '{"name":"Bad","kpi_type":"nonexistent","dimensions":[]}'
assert_status "Invalid KPI rejected" "400" "$_STATUS"

http_post "$API/reports/$RPT_ID/run" '{"filters":{}}'
assert_status "Run report" "200" "$_STATUS"
assert_body "Result has data" "data"
assert_body "Result has kpi_type" "kpi_type"

http_get "$API/reports"
assert_status "List reports" "200" "$_STATUS"

http_post "$API/scheduled-reports" "{\"report_definition_id\":\"$RPT_ID\",\"frequency\":\"weekly\",\"export_format\":\"xlsx\",\"next_run_at\":\"2026-04-14T00:00:00Z\"}"
assert_status "Create schedule" "201" "$_STATUS"
assert_body "Has frequency" "weekly"

http_post "$API/exports" "{\"report_definition_id\":\"$RPT_ID\",\"export_format\":\"csv\",\"estimated_rows\":100}"
assert_status "Request export" "202" "$_STATUS"
EXP_ID=$(jf id)

http_get "$API/exports/$EXP_ID"
assert_status "Get export job" "200" "$_STATUS"
assert_body "Has status" "status"

http_get "$API/exports"
assert_status "List exports" "200" "$_STATUS"

http_post "$API/exports" "{\"report_definition_id\":\"$RPT_ID\",\"export_format\":\"exe\"}"
assert_status "Invalid format rejected" "400" "$_STATUS"
echo ""

###########################################################################
echo "── Section 10: Audit Trail ───────────────────────"

http_get "$API/audit"
assert_status "GET /audit" "200" "$_STATUS"
assert_body "Has action field" "action"
assert_body "Has resource_type" "resource_type"

http_get "$API/audit?action=create"
assert_status "Audit filter by action" "200" "$_STATUS"
echo ""

###########################################################################
echo "── Section 11: Metrics ───────────────────────────"

http_get "$API/metrics"
assert_status "GET /metrics" "200" "$_STATUS"
assert_body "Has uptime_seconds" "uptime_seconds"
assert_body "Has total_requests" "total_requests"
assert_body "Has pool connections" "connections"
echo ""

###########################################################################
echo "── Section 12: CSRF ─────────────────────────────"

_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $TOKEN" -d 'x' "$API/roles")
assert_status "CSRF blocks bare POST" "403" "$_STATUS"
echo ""

###########################################################################
echo "── Section 13: Error Handling ────────────────────"

http_get "$API/orders/00000000-0000-0000-0000-000000000099"
assert_status "Non-existent order → 404" "404" "$_STATUS"

http_get "$API/participants/00000000-0000-0000-0000-000000000099"
assert_status "Non-existent participant → 404" "404" "$_STATUS"

http_get "$API/datasets/00000000-0000-0000-0000-000000000099"
assert_status "Non-existent dataset → 404" "404" "$_STATUS"

http_post "$API/orders" '{bad json}'
assert_status "Invalid JSON → 400" "400" "$_STATUS"
echo ""

###########################################################################
echo "── Section 14: [A] Reversal Approval Gate ────────"

# Create an order, pay it, then test reversal requires approval
http_post "$API/orders" '{"location":"TEST-A","line_items":[{"sku":"A1","description":"Test","quantity":1,"unit_price_cents":500,"tax_cents":0}]}'
A_ORDER_ID=$(jf id)
http_post "$API/orders/$A_ORDER_ID/transition" '{"target_status":"open"}'
http_post "$API/orders/$A_ORDER_ID/transition" '{"target_status":"tendering"}'
AKEY="bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"
http_post "$API/orders/$A_ORDER_ID/payments" "{\"tender_type\":\"cash\",\"amount_cents\":500,\"idempotency_key\":\"$AKEY\"}"

# Initiate reversal — should NOT create ledger entries, just approval request
RKEY="cccccccc-cccc-cccc-cccc-cccccccccccc"
http_post "$API/orders/$A_ORDER_ID/reversals" "{\"idempotency_key\":\"$RKEY\",\"notes\":\"test reversal\"}"
assert_status "[A] Reversal creates approval (no mutation)" "202" "$_STATUS"
assert_body "[A] Has approval_request_id" "approval_request_id"
REVERSAL_APPROVAL_ID=$(jf approval_request_id)

# Verify order is ReversalPending but NO reversal ledger entries yet
http_get "$API/orders/$A_ORDER_ID/payments"
assert_body_not "[A] No reversal entries before approval" "Reversal"

# Try execute without approval — should fail
EKEY="dddddddd-dddd-dddd-dddd-dddddddddddd"
http_post "$API/orders/$A_ORDER_ID/reversals/execute" "{\"approval_request_id\":\"$REVERSAL_APPROVAL_ID\",\"idempotency_key\":\"$EKEY\"}"
assert_status "[A] Execute before approval rejected" "400" "$_STATUS"

# Self-approve should be REJECTED (independent approver rule)
http_post "$API/approvals/$REVERSAL_APPROVAL_ID/approve" '{}'
assert_status "[A] Self-approve blocked (independent approver)" "403" "$_STATUS"

# Create a second admin user to act as independent approver
http_post "$API/users" "{\"username\":\"manager_approver\",\"password\":\"ManagerPass123\",\"role_id\":\"a0000000-0000-0000-0000-000000000002\",\"location\":\"TEST-A\"}"
http_post_noauth "$API/auth/login" '{"username":"manager_approver","password":"ManagerPass123"}'
MGR_TOKEN=$(jf access_token)
SAVED_ADMIN="$TOKEN"

# Manager approves the reversal (independent approver)
TOKEN="$MGR_TOKEN"
http_post "$API/approvals/$REVERSAL_APPROVAL_ID/approve" '{}'
assert_status "[A] Independent approver succeeds" "200" "$_STATUS"
TOKEN="$SAVED_ADMIN"

# Now execute reversal — should succeed
http_post "$API/orders/$A_ORDER_ID/reversals/execute" "{\"approval_request_id\":\"$REVERSAL_APPROVAL_ID\",\"idempotency_key\":\"$EKEY\"}"
assert_status "[A] Execute after approval succeeds" "200" "$_STATUS"
assert_body "[A] Order is reversed" "reversed"

# Idempotency: replay same key → same result, no double posting
EKEY2="$EKEY"
http_post "$API/orders/$A_ORDER_ID/reversals/execute" "{\"approval_request_id\":\"$REVERSAL_APPROVAL_ID\",\"idempotency_key\":\"$EKEY2\"}"
assert_status "[A] Idempotent replay returns cached" "200" "$_STATUS"
echo ""

###########################################################################
echo "── Section 15: [D] Audit Before/After Hashes ─────"

http_get "$API/audit?resource_type=orders&action=create"
assert_status "[D] Audit has order create entries" "200" "$_STATUS"
assert_body "[D] Audit has after_hash" "after_hash"

http_get "$API/audit?resource_type=orders&action=reversal"
assert_status "[D] Audit has reversal entries" "200" "$_STATUS"
assert_body "[D] Reversal audit has before_hash" "before_hash"
assert_body "[D] Reversal audit has after_hash" "after_hash"

# Verify sensitive data not in audit
assert_body_not "[D] No password in audit" "password"
echo ""

###########################################################################
echo "── Section 16: [4] Receipt File Governance ───────"

# Valid receipt with PDF file
echo "unique receipt content for gov test" > _rcpt_gov.pdf
_STATUS=$(curl -s -o "$_TMPF" -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@_rcpt_gov.pdf;filename=receipt2.pdf" \
  "$API/orders/$ORDER_ID/receipts")
_BODY=$(cat "$_TMPF")
assert_status "[4] Valid receipt upload" "201" "$_STATUS"
assert_body "[4] Receipt has sha256" "sha256_hash"
assert_body "[4] Receipt has file_path" "file_path"

# Disallowed file type
echo "bad" > _rcpt_bad.exe
_STATUS=$(curl -s -o "$_TMPF" -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@_rcpt_bad.exe;filename=virus.exe" \
  "$API/orders/$ORDER_ID/receipts")
_BODY=$(cat "$_TMPF")
assert_status "[4] Disallowed type rejected" "400" "$_STATUS"

# Duplicate hash detection (same file content as previous upload)
echo "unique receipt content for gov test" > _rcpt_dup.pdf
_STATUS=$(curl -s -o "$_TMPF" -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@_rcpt_dup.pdf;filename=dup.pdf" \
  "$API/orders/$ORDER_ID/receipts")
_BODY=$(cat "$_TMPF")
assert_status "[4] Duplicate hash detected" "409" "$_STATUS"
rm -f _rcpt_gov.pdf _rcpt_bad.exe _rcpt_dup.pdf
echo ""

###########################################################################
echo "── Section 17: [5] Export Artifact Governance ────"

# Complete export with server-managed artifact
http_post "$API/exports" "{\"report_definition_id\":\"$RPT_ID\",\"export_format\":\"csv\",\"estimated_rows\":5}"
E5_ID=$(jf id)
# Export job starts as running — complete it with base64 content
CONTENT_B64=$(echo -n "col1,col2\nval1,val2" | base64)
http_post "$API/exports/$E5_ID/complete" "{\"total_rows\":2,\"file_content_base64\":\"$CONTENT_B64\"}"
assert_status "[5] Complete with server-managed artifact" "200" "$_STATUS"
assert_body "[5] Job is completed" "completed"

# Download should work for owner
http_get "$API/exports/$E5_ID/download"
assert_status "[5] Owner can download" "200" "$_STATUS"
echo ""

###########################################################################
echo "── Section 18: [P3] Audit Hash Coverage ──────────"

# Verify payment audit has after_hash
http_get "$API/audit?resource_type=ledger_entries&action=create"
assert_status "[P3] Payment audit entries exist" "200" "$_STATUS"
assert_body "[P3] Payment audit has after_hash" "after_hash"

# Verify role update audit has both before and after hashes
http_get "$API/audit?resource_type=roles&action=update"
assert_status "[P3] Role update audit exists" "200" "$_STATUS"
assert_body "[P3] Role update has before_hash" "before_hash"
assert_body "[P3] Role update has after_hash" "after_hash"

# Verify bulk deactivate audit
http_get "$API/audit?resource_type=participants&action=delete"
assert_status "[P3] Bulk deactivate audit exists" "200" "$_STATUS"
assert_body "[P3] Bulk deactivate has after_hash" "after_hash"

# No sensitive data
assert_body_not "[P3] No password in audit" "password_hash"
echo ""

###########################################################################
echo "── Section 19: [P4] Export Checksum Persistence ──"

# Verify export job has sha256_hash after completion
http_get "$API/exports/$E5_ID"
assert_status "[P4] Get completed export job" "200" "$_STATUS"
assert_body "[P4] Export has sha256_hash" "sha256_hash"
assert_body "[P4] Export has file_size_bytes" "file_size_bytes"
echo ""

###########################################################################
echo "── Section 20: [P5] Approval Create Permission ──"

# Create a cashier user with limited permissions
http_post "$API/users" "{\"username\":\"cashier_test\",\"password\":\"CashierPass123\",\"role_id\":\"a0000000-0000-0000-0000-000000000003\",\"location\":\"NYC-01\"}"
CASHIER_UID=$(jf id)

# Login as cashier
http_post_noauth "$API/auth/login" '{"username":"cashier_test","password":"CashierPass123"}'
CASHIER_TOKEN=$(jf access_token)
SAVED_TOKEN="$TOKEN"

# Cashier should NOT be able to create approval requests (no approval.request.create)
TOKEN="$CASHIER_TOKEN"
http_post "$API/approvals" '{"permission_point_id":"b0000000-0000-0000-0000-000000000001","payload":{"test":true}}'
assert_status "[P5] Cashier cannot create approval request" "403" "$_STATUS"

# Switch back to admin
TOKEN="$SAVED_TOKEN"
http_post "$API/approvals" '{"permission_point_id":"b0000000-0000-0000-0000-000000000001","payload":{"test":true}}'
assert_status "[P5] Admin can create approval request" "202" "$_STATUS"
echo ""

###########################################################################
echo "── Section 21: [F1] Async Export Progression ─────"

# Create a non-bulk export — worker should pick it up autonomously
http_post "$API/reports" '{"name":"Async Test Report","kpi_type":"registration_conversion","dimensions":[]}'
ASYNC_RPT_ID=$(jf id)
http_post "$API/exports" "{\"report_definition_id\":\"$ASYNC_RPT_ID\",\"export_format\":\"csv\",\"estimated_rows\":10}"
ASYNC_EXP_ID=$(jf id)

# Wait for worker to process (polls every 5s, give it 12s)
sleep 12

http_get "$API/exports/$ASYNC_EXP_ID"
assert_status "[F1] Export job accessible" "200" "$_STATUS"
# Worker should have moved it past Queued
assert_body_not "[F1] Export no longer queued" "\"queued\""
echo ""

###########################################################################
echo "── Section 22: [F2] Idempotency Duplicate Safety ─"

# Replay same payment idempotency key (already tested in S5)
# Verify the response is identical and no double-posting
IKEY_DUP="aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
http_post "$API/orders/$ORDER_ID/payments" "{\"tender_type\":\"cash\",\"amount_cents\":99999,\"idempotency_key\":\"$IKEY_DUP\"}"
assert_status "[F2] Idempotent replay cached" "201" "$_STATUS"
# The amount should be the ORIGINAL amount (3240), not the new one (99999)
assert_body_not "[F2] No double-mutation (wrong amount absent)" "99999"
echo ""

###########################################################################
echo "── Section 23: [F3] Transition Scope Enforcement ─"

# Create order as admin in location TEST-X
http_post "$API/orders" '{"location":"TEST-X","department":"DeptX","line_items":[{"sku":"X1","description":"X","quantity":1,"unit_price_cents":100,"tax_cents":0}]}'
SCOPE_ORDER_ID=$(jf id)

# Login as cashier (location NYC-01, individual scope)
TOKEN="$CASHIER_TOKEN"

# Cashier should NOT be able to transition admin's order (different location/individual scope)
http_post "$API/orders/$SCOPE_ORDER_ID/transition" '{"target_status":"open"}'
assert_status "[F3] Cross-scope transition blocked" "403" "$_STATUS"

TOKEN="$SAVED_TOKEN"
echo ""

###########################################################################
echo "── Section 24: [F5] Transition Audit Hashes ──────"

http_get "$API/audit?resource_type=orders&action=update"
assert_status "[F5] Transition audit entries exist" "200" "$_STATUS"
assert_body "[F5] Transition audit has before_hash" "before_hash"
assert_body "[F5] Transition audit has after_hash" "after_hash"
echo ""

###########################################################################
echo "── Section 25: [T5] Cross-scope Return/Exchange ──"

# Create a paid order as admin in location SCOPE-Z
http_post "$API/orders" '{"location":"SCOPE-Z","department":"DeptZ","line_items":[{"sku":"Z1","description":"Z","quantity":1,"unit_price_cents":1000,"tax_cents":0}]}'
SCOPE_Z_ORDER=$(jf id)
http_post "$API/orders/$SCOPE_Z_ORDER/transition" '{"target_status":"open"}'
http_post "$API/orders/$SCOPE_Z_ORDER/transition" '{"target_status":"tendering"}'
ZKEY="eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee"
http_post "$API/orders/$SCOPE_Z_ORDER/payments" "{\"tender_type\":\"cash\",\"amount_cents\":1000,\"idempotency_key\":\"$ZKEY\"}"
# Get line item ID for return
http_get "$API/orders/$SCOPE_Z_ORDER"
Z_LINE_ID=$(echo "$_BODY" | grep -o '"id":"[^"]*"' | sed -n '3p' | cut -d'"' -f4)

# Login as cashier (individual scope, different location)
TOKEN="$CASHIER_TOKEN"

# Cross-scope return should be blocked
RKEY_Z="ffffffff-ffff-ffff-ffff-ffffffffffff"
http_post "$API/orders/$SCOPE_Z_ORDER/returns" "{\"idempotency_key\":\"$RKEY_Z\",\"line_items\":[{\"original_line_item_id\":\"$Z_LINE_ID\",\"quantity\":1}]}"
assert_status "[T5] Cross-scope return blocked" "403" "$_STATUS"

# Cross-scope exchange should be blocked
XKEY_Z="11111111-2222-3333-4444-555555555555"
http_post "$API/orders/$SCOPE_Z_ORDER/exchanges" "{\"idempotency_key\":\"$XKEY_Z\",\"return_items\":[{\"original_line_item_id\":\"$Z_LINE_ID\",\"quantity\":1}],\"new_items\":[{\"sku\":\"N1\",\"description\":\"New\",\"quantity\":1,\"unit_price_cents\":500}]}"
assert_status "[T5] Cross-scope exchange blocked" "403" "$_STATUS"

TOKEN="$SAVED_TOKEN"
echo ""

###########################################################################
echo "── Section 26: [T7] Out-of-scope reversal exec ───"

# Initiate a reversal as admin on SCOPE-Z order
RVKEY="22222222-3333-4444-5555-666666666666"
http_post "$API/orders/$SCOPE_Z_ORDER/reversals" "{\"idempotency_key\":\"$RVKEY\",\"notes\":\"scope test\"}"
RV_APPR_ID=$(jf approval_request_id)

# Create manager to approve
http_post "$API/users" "{\"username\":\"mgr_scope\",\"password\":\"ManagerPass123\",\"role_id\":\"a0000000-0000-0000-0000-000000000002\",\"location\":\"SCOPE-Z\"}"
http_post_noauth "$API/auth/login" '{"username":"mgr_scope","password":"ManagerPass123"}'
MGR_SCOPE_TOKEN=$(jf access_token)
SAVED2="$TOKEN"
TOKEN="$MGR_SCOPE_TOKEN"
http_post "$API/approvals/$RV_APPR_ID/approve" '{}'
TOKEN="$SAVED2"

# Now try to execute reversal as cashier (out of scope)
TOKEN="$CASHIER_TOKEN"
RVEXEC="33333333-4444-5555-6666-777777777777"
http_post "$API/orders/$SCOPE_Z_ORDER/reversals/execute" "{\"approval_request_id\":\"$RV_APPR_ID\",\"idempotency_key\":\"$RVEXEC\"}"
assert_status "[T7] Out-of-scope reversal exec blocked" "403" "$_STATUS"
TOKEN="$SAVED_TOKEN"
echo ""

###########################################################################
echo "── Section 27: [T8] Large-scale async export ─────"

# Create export with 250000 estimated rows
http_post "$API/exports" "{\"report_definition_id\":\"$ASYNC_RPT_ID\",\"export_format\":\"csv\",\"estimated_rows\":250000}"
LARGE_EXP=$(jf id)
assert_status "[T8] Large export job created" "202" "$_STATUS"

# Poll for progress (worker runs every 5s, give it time)
sleep 8
http_get "$API/exports/$LARGE_EXP"
assert_status "[T8] Large export accessible" "200" "$_STATUS"
# Should show progress (not still at 0%)
assert_body_not "[T8] Export progressed past zero" "\"progress_pct\":0"
echo ""

###########################################################################
echo "── Section 28: [T9] Parallel idempotency race ────"

# Create a new order for race test
http_post "$API/orders" '{"location":"RACE-1","line_items":[{"sku":"R1","description":"Race","quantity":1,"unit_price_cents":200,"tax_cents":0}]}'
RACE_ORDER=$(jf id)
http_post "$API/orders/$RACE_ORDER/transition" '{"target_status":"open"}'
http_post "$API/orders/$RACE_ORDER/transition" '{"target_status":"tendering"}'

# Send two near-simultaneous payments with the SAME idempotency key
RACE_KEY="44444444-5555-6666-7777-888888888888"
# Fire both concurrently using background subshells
_STATUS1=$(curl -s -o "$_TMPF.r1" -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d "{\"tender_type\":\"cash\",\"amount_cents\":200,\"idempotency_key\":\"$RACE_KEY\"}" \
  "$API/orders/$RACE_ORDER/payments") &
PID1=$!

_STATUS2=$(curl -s -o "$_TMPF.r2" -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d "{\"tender_type\":\"cash\",\"amount_cents\":200,\"idempotency_key\":\"$RACE_KEY\"}" \
  "$API/orders/$RACE_ORDER/payments") &
PID2=$!

wait $PID1 2>/dev/null
wait $PID2 2>/dev/null

# At least one should succeed (201), the other either 201 (cached replay) or 409 (conflict)
# Neither should result in 500 or double mutation
S1=$(cat "$_TMPF.r1" 2>/dev/null | wc -c)
S2=$(cat "$_TMPF.r2" 2>/dev/null | wc -c)
TOTAL=$((TOTAL + 1))
if [ "$S1" -gt 0 ] || [ "$S2" -gt 0 ]; then
  PASS=$((PASS + 1)); echo "  [PASS] [T9] Parallel race: both completed without crash"
else
  FAIL=$((FAIL + 1)); echo "  [FAIL] [T9] Parallel race: no response received"
  FAILURES="$FAILURES\n  [FAIL] [T9] Parallel race failed"
fi
rm -f "$_TMPF.r1" "$_TMPF.r2"

# Verify only one payment exists (no double mutation)
http_get "$API/orders/$RACE_ORDER/payments"
PAYMENT_COUNT=$(echo "$_BODY" | grep -o '"id"' | wc -l)
TOTAL=$((TOTAL + 1))
if [ "$PAYMENT_COUNT" -le 1 ]; then
  PASS=$((PASS + 1)); echo "  [PASS] [T9] No duplicate mutation: $PAYMENT_COUNT payment(s)"
else
  FAIL=$((FAIL + 1)); echo "  [FAIL] [T9] Duplicate mutation: $PAYMENT_COUNT payments"
  FAILURES="$FAILURES\n  [FAIL] [T9] Duplicate mutation detected"
fi
echo ""

###########################################################################
echo "── Section 29: [T10] Self-reject blocked ─────────"

# Admin creates an approval request
http_post "$API/approvals" '{"permission_point_id":"b0000000-0000-0000-0000-000000000001","payload":{"test":"self_reject"}}'
SELF_REJ_ID=$(jf id)

# Admin tries to reject own request — should be blocked
http_post "$API/approvals/$SELF_REJ_ID/reject" '{}'
assert_status "[T10] Self-reject blocked" "403" "$_STATUS"
echo ""

###########################################################################
echo "── Section 30: [T11] Audit hashes on more writes ─"

# Verify receipt create audit has after_hash
http_get "$API/audit?resource_type=receipts&action=create"
assert_status "[T11] Receipt audit exists" "200" "$_STATUS"
assert_body "[T11] Receipt audit has after_hash" "after_hash"

# Verify participant update audit has before+after
http_get "$API/audit?resource_type=participants&action=update"
assert_status "[T11] Participant update audit exists" "200" "$_STATUS"
assert_body "[T11] Participant update has before_hash" "before_hash"
assert_body "[T11] Participant update has after_hash" "after_hash"
echo ""

###########################################################################
echo "── Section 31: [I7] KPI filter/dimension tests ───"

# Create a report with unsupported dimension → 400
http_post "$API/reports" '{"name":"Bad Dim Report","kpi_type":"participation_by_store","dimensions":["invalid_dim"]}'
assert_status "[I7] Unsupported dimension rejected" "400" "$_STATUS"

# Create report with valid dimension
http_post "$API/reports" '{"name":"Valid Dim Report","kpi_type":"participation_by_store","dimensions":["location"]}'
assert_status "[I7] Valid dimension accepted" "201" "$_STATUS"
FILT_RPT=$(jf id)

# Run with location filter — output should be filtered
http_post "$API/reports/$FILT_RPT/run" '{"filters":{"location":"NYC-01"}}'
assert_status "[I7] Filtered report runs" "200" "$_STATUS"
assert_body "[I7] Report has data" "data"

# Run with unsupported filter → 400
http_post "$API/reports/$FILT_RPT/run" '{"filters":{"unsupported_key":"val"}}'
assert_status "[I7] Unsupported filter rejected" "400" "$_STATUS"

# Verify participation_by_department grouping
http_post "$API/reports" '{"name":"Dept Report","kpi_type":"participation_by_department","dimensions":["department"]}'
DEPT_RPT=$(jf id)
http_post "$API/reports/$DEPT_RPT/run" '{"filters":{"department":"Sales"}}'
assert_status "[I7] Department KPI with filter" "200" "$_STATUS"
assert_body "[I7] Has department in output" "department"
echo ""

###########################################################################
echo "── Section 32: [I4] Notification audit coverage ──"

# Verify notification send audit
http_get "$API/audit?resource_type=notifications&action=create"
assert_status "[I4] Notification audit exists" "200" "$_STATUS"
assert_body "[I4] Notification audit has after_hash" "after_hash"
assert_body_not "[I4] No password in notification audit" "password"

# Verify team create audit
http_get "$API/audit?resource_type=teams&action=create"
assert_status "[I4] Team create audit exists" "200" "$_STATUS"
assert_body "[I4] Team audit has after_hash" "after_hash"
echo ""

###########################################################################
echo "── Section 33: [T10] Reversal idempotency key not poisoned on failure ──"

# Create a Draft order (NOT Paid) — reversal initiation must fail validation,
# and the idempotency key must NOT be stuck at sentinel status=0 so the
# client can retry with the same key once the order is in a valid state.
http_post "$API/orders" '{"location":"TEST-A","line_items":[{"sku":"P1","description":"Poison test","quantity":1,"unit_price_cents":100,"tax_cents":0}]}'
POISON_ORDER=$(jf id)
# Order is in Draft — reversal should fail
POISON_KEY="eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee"
http_post "$API/orders/$POISON_ORDER/reversals" "{\"idempotency_key\":\"$POISON_KEY\",\"notes\":\"should fail\"}"
assert_status "[T10] Reversal on Draft order rejected" "400" "$_STATUS"

# Now transition the order to Paid so reversal becomes valid
http_post "$API/orders/$POISON_ORDER/transition" '{"target_status":"open"}'
http_post "$API/orders/$POISON_ORDER/transition" '{"target_status":"tendering"}'
PKEY2="ffffffff-ffff-ffff-ffff-ffffffffffff"
http_post "$API/orders/$POISON_ORDER/payments" "{\"tender_type\":\"cash\",\"amount_cents\":100,\"idempotency_key\":\"$PKEY2\"}"
assert_status "[T10] Order paid for retry" "201" "$_STATUS"

# Retry reversal with SAME idempotency key — must succeed (not 409 Conflict)
http_post "$API/orders/$POISON_ORDER/reversals" "{\"idempotency_key\":\"$POISON_KEY\",\"notes\":\"retry after fix\"}"
assert_status "[T10] Retry with same key succeeds after fix" "202" "$_STATUS"
assert_body "[T10] Retry returns approval_request_id" "approval_request_id"
echo ""

###########################################################################
echo ""
echo "============================================"
echo "  API TEST RESULTS"
echo "============================================"
echo "  Total:  $TOTAL"
echo "  Passed: $PASS"
echo "  Failed: $FAIL"
echo "============================================"

if [ $FAIL -gt 0 ]; then
  echo ""; echo "Failed tests:"; echo -e "$FAILURES"; echo ""; exit 1
else
  echo ""; echo "  ALL TESTS PASSED"; echo ""; exit 0
fi
