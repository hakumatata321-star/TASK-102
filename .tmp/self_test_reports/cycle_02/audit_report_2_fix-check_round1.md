# RetailOps Static Audit Report (Fix Check Round 1)

## 1. Verdict
- **Overall conclusion: Partial Pass**
- The prior blocker areas were materially improved (independent approver rule, object-scope checks added on return/exchange/reversal, request-aware RBAC added in several critical paths, targeted tests updated), but several high-risk gaps still prevent full pass.

## 2. Scope and Static Verification Boundary
- **Reviewed:** updated security and governance paths in `approval_handler`, `return_handler`, `register_handler`, `dataset_handler`, `order_handler`, `rbac/guard`, capability seed migrations, README security notes, and API test script changes.
- **Not reviewed exhaustively:** every handler for full-line coverage of all non-critical flows.
- **Intentionally not executed:** app runtime, Docker, migrations, tests, load/perf checks.
- **Manual verification required:** p95 latency under 300ms @ 200 users, real async export throughput to 250k rows, runtime race-condition behavior for idempotency.

## 3. Repository / Requirement Mapping Summary
- **Prompt core objective:** governed offline RetailOps backend with strict auth/RBAC/data-scope, POS state/idempotency/reconciliation controls, dataset lineage/rollback governance, attachment integrity, approval gates, exports, and auditability.
- **Mapped implementation updates:**
  - Independent approver enforcement in approvals.
  - POS return/exchange/reversal scope enforcement and selected request-aware RBAC.
  - Dataset rollback execute path moved to request-aware capability check.
  - API capability seeding expanded for critical endpoints.
  - API tests now include self-approval rejection flow.

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability
- **Conclusion: Pass**
- **Rationale:** startup/config/test docs remain usable; updated security behavior is documented.
- **Evidence:** `repo/README.md:10`, `repo/README.md:45`, `repo/README.md:86`.

#### 1.2 Material deviation from prompt
- **Conclusion: Partial Pass**
- **Rationale:** earlier major deviations reduced, but full requirement fit is still incomplete due to async export simulation and residual control gaps.
- **Evidence:** `repo/src/handlers/approval_handler.rs:64`, `repo/src/handlers/return_handler.rs:63`, `repo/src/handlers/export_handler.rs:95`.

### 2. Delivery Completeness

#### 2.1 Core requirement coverage
- **Conclusion: Partial Pass**
- **Rationale:** broad functional coverage exists and critical security/governance fixes were added; still incomplete on some strict non-functional/control semantics.
- **Evidence:** `repo/src/routes.rs:79`, `repo/src/routes.rs:88`, `repo/src/routes.rs:192`, `repo/src/routes.rs:319`.

#### 2.2 0→1 end-to-end deliverable
- **Conclusion: Partial Pass**
- **Rationale:** codebase is full-service shaped, but export job execution is still explicitly simulated rather than true asynchronous processing.
- **Evidence:** `repo/src/handlers/export_handler.rs:94`, `repo/src/handlers/export_handler.rs:95`, `repo/src/handlers/export_handler.rs:97`.

### 3. Engineering and Architecture Quality

#### 3.1 Structure and modularity
- **Conclusion: Pass**
- **Rationale:** architecture remains clearly decomposed by domain and cross-cutting modules.
- **Evidence:** `repo/src/main.rs:4`, `repo/src/routes.rs:3`, `repo/src/handlers/mod.rs:1`.

#### 3.2 Maintainability/extensibility
- **Conclusion: Partial Pass**
- **Rationale:** maintainability improved with focused security fixes, but enforcement style is still mixed across handlers (request-aware vs non-request-aware checks).
- **Evidence:** `repo/src/handlers/return_handler.rs:51`, `repo/src/handlers/register_handler.rs:24`, `repo/src/handlers/register_handler.rs:199`.

### 4. Engineering Details and Professionalism

#### 4.1 Error handling/logging/validation/API detail
- **Conclusion: Partial Pass**
- **Rationale:** validations and error mapping are solid; audit hashing improved in critical paths; logging remains plain text access log rather than clearly structured JSON.
- **Evidence:** `repo/src/errors.rs:40`, `repo/src/handlers/register_handler.rs:230`, `repo/src/main.rs:49`.

#### 4.2 Product-level readiness
- **Conclusion: Partial Pass**
- **Rationale:** service behaves like a real product backend statically, but a few core governance/NFR items still appear simplified.
- **Evidence:** `repo/src/handlers/export_handler.rs:95`, `repo/src/handlers/order_handler.rs:330`.

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business goal and constraint fit
- **Conclusion: Partial Pass**
- **Rationale:** requirement understanding is now significantly better (independent approval + POS scope enforcement + capability seeding). Remaining mismatches prevent full pass.
- **Evidence:** `repo/src/handlers/approval_handler.rs:65`, `repo/src/handlers/return_handler.rs:64`, `repo/migrations/00000000000044_seed_critical_api_capabilities/up.sql:4`, `repo/src/handlers/export_handler.rs:95`.

### 6. Aesthetics (frontend-only)
- **Conclusion: Not Applicable**
- **Rationale:** backend-only task/repo.
- **Evidence:** `repo/src/main.rs:1`, `repo/Cargo.toml:7`.

## 5. Issues / Suggestions (Severity-Rated)

### High

1) **Export generation is still simulated, not true async processing**
- **Conclusion:** Fail
- **Evidence:** `repo/src/handlers/export_handler.rs:95`, `repo/src/handlers/export_handler.rs:97`
- **Impact:** prompt requires asynchronous export generation with progress for up to 250,000 rows; current flow still depends on simulated start + admin completion endpoints.
- **Minimum actionable fix:** implement in-process worker/job runner that picks queued jobs and performs generation/progress updates automatically.

2) **Idempotency handling remains non-atomic (race risk on duplicate writes)**
- **Conclusion:** Fail
- **Evidence:** `repo/src/pos/idempotency.rs:12`, `repo/src/pos/idempotency.rs:41`, `repo/src/handlers/return_handler.rs:54`, `repo/src/handlers/return_handler.rs:172`
- **Impact:** concurrent duplicate submissions can both pass `check_idempotency` before persistence, risking duplicate stock/accounting mutation.
- **Minimum actionable fix:** reserve idempotency key atomically before mutation in same DB transaction (insert-first / lock strategy), then finalize response record.

3) **Object-level scope still missing in `transition_order` mutation path**
- **Conclusion:** Fail
- **Evidence:** `repo/src/handlers/order_handler.rs:330`, `repo/src/handlers/order_handler.rs:332`, `repo/src/handlers/order_handler.rs:372`
- **Impact:** user with `order.transition` permission may transition out-of-scope orders because scope context is not enforced against target order.
- **Minimum actionable fix:** apply `ctx.enforce_scope(order.cashier_user_id, order.department.as_deref(), Some(&order.location))` before performing transition.

### Medium

4) **Layer-3 RBAC is improved but still inconsistent for some critical routes**
- **Conclusion:** Partial Fail
- **Evidence:** `repo/src/handlers/return_handler.rs:51`, `repo/src/handlers/register_handler.rs:24`, `repo/src/handlers/approval_handler.rs:53`, `repo/src/handlers/dataset_handler.rs:431`
- **Impact:** method/path capability enforcement is not uniformly guaranteed across all protected endpoints.
- **Minimum actionable fix:** standardize critical handlers on `check_permission_for_request` where request context exists.

5) **Audit before/after hash coverage is improved but not universal for all writes**
- **Conclusion:** Partial Fail
- **Evidence:** `repo/src/handlers/return_handler.rs:532`, `repo/src/handlers/register_handler.rs:245`, `repo/src/audit/middleware.rs:110`, `repo/src/audit/middleware.rs:111`
- **Impact:** some write operations may still rely on middleware entries without before/after hashes, weakening full-forensic requirement.
- **Minimum actionable fix:** ensure every write endpoint emits explicit `audit_write` with safe before/after payload.

6) **Structured logging requirement remains only partially satisfied**
- **Conclusion:** Partial Fail
- **Evidence:** `repo/src/main.rs:49`, `repo/src/main.rs:50`
- **Impact:** logs are human-readable but not strongly structured for deterministic local querying/automation.
- **Minimum actionable fix:** adopt JSON log formatter with stable fields (timestamp, req_id, user_id, path, status, latency, error_code).

## 6. Security Review Summary

- **authentication entry points — Pass**
  - Username/password + lockout architecture still present.
  - Evidence: `repo/src/handlers/auth_handler.rs:46`, `repo/src/auth/lockout.rs:28`.

- **route-level authorization — Partial Pass**
  - Request-aware checks now present in several critical endpoints; still not universal.
  - Evidence: `repo/src/handlers/return_handler.rs:51`, `repo/src/handlers/dataset_handler.rs:431`, `repo/src/handlers/register_handler.rs:24`.

- **object-level authorization — Partial Pass**
  - Fixed in return/exchange/reversal handlers; still absent in `transition_order` path.
  - Evidence: `repo/src/handlers/return_handler.rs:64`, `repo/src/handlers/return_handler.rs:484`, `repo/src/handlers/order_handler.rs:330`.

- **function-level authorization — Pass**
  - Independent approver control now enforced in approval decision endpoints.
  - Evidence: `repo/src/handlers/approval_handler.rs:65`, `repo/src/handlers/approval_handler.rs:129`.

- **tenant/user data isolation — Partial Pass**
  - Scope checks are strong in many handlers; remaining mutation gaps keep this partial.
  - Evidence: `repo/src/handlers/order_handler.rs:419`, `repo/src/handlers/participant_handler.rs:148`, `repo/src/handlers/order_handler.rs:330`.

- **admin/internal/debug protection — Partial Pass**
  - Admin-style endpoints are permission-checked; full adversarial matrix not statically proven.
  - Evidence: `repo/src/handlers/export_handler.rs:176`, `repo/src/handlers/audit_handler.rs:17`.

## 7. Tests and Logging Review

- **Unit tests — Partial Pass**
  - Utility/security logic tests exist; some newly fixed authorization invariants still rely mainly on API script coverage.
  - Evidence: `repo/unit_tests/README.md:7`, `repo/src/rbac/guard.rs:154`.

- **API/integration tests — Partial Pass (improved)**
  - Independent approver rule is now covered in API test flow.
  - Cross-scope mutation abuse tests are still missing.
  - Evidence: `repo/API_tests/run_api_tests.sh:464`, `repo/API_tests/run_api_tests.sh:466`, `repo/API_tests/run_api_tests.sh:450`.

- **Logging categories/observability — Partial Pass**
  - Health/metrics endpoints and request metrics exist; log structure still basic format string.
  - Evidence: `repo/src/observability/health.rs:38`, `repo/src/observability/metrics.rs:8`, `repo/src/main.rs:49`.

- **Sensitive-data leakage risk in logs/responses — Partial Pass**
  - Masking and no-password response behavior remain; runtime log payload hygiene still requires manual check.
  - Evidence: `repo/src/models/user.rs:58`, `repo/API_tests/run_api_tests.sh:171`, `repo/src/errors.rs:81`.

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests are Rust `#[cfg(test)]`; API tests are bash/curl assertions.
- Test commands are documented.
- Evidence: `repo/unit_tests/README.md:3`, `repo/unit_tests/run_unit_tests.sh:31`, `repo/API_tests/run_api_tests.sh:1`, `repo/README.md:47`.

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Independent approver rule | `repo/API_tests/run_api_tests.sh:464` | self-approve -> 403 | sufficient | Reject path not similarly asserted | Add self-reject 403 test |
| Reversal approval lifecycle | `repo/API_tests/run_api_tests.sh:450`, `repo/API_tests/run_api_tests.sh:461`, `repo/API_tests/run_api_tests.sh:481` | initiate 202, execute blocked pre-approval, succeeds post-approval | basically covered | No cross-scope actor test | Add reversal execute attempt by out-of-scope user |
| Return/exchange object scope enforcement | No explicit tests found | N/A | missing | Current suite does not verify cross-scope 403 for `/returns` and `/exchanges` | Add multi-user cross-location return/exchange negative tests |
| Audit hashes for critical writes | `repo/API_tests/run_api_tests.sh:498`, `repo/API_tests/run_api_tests.sh:567` | before/after hash field checks in selected resources | basically covered | Not comprehensive for all writes | Add targeted matrix over key write routes |
| API capability layer-3 behavior | No direct negative tests for method/path mismatch | N/A | insufficient | Could regress silently despite seed data | Add tests for capability mismatch returning 403 |
| Idempotency race safety | Replay-only checks at `repo/API_tests/run_api_tests.sh:487` | same-key replay expected 200 | insufficient | No concurrency/race validation | Add parallel duplicate-submit test |
| Async export autonomy | `repo/API_tests/run_api_tests.sh:545` + manual complete at `repo/API_tests/run_api_tests.sh:549` | job manually completed via admin endpoint | insufficient | Does not verify autonomous async worker | Add test expecting queued/running job progression without manual complete |

### 8.3 Security Coverage Audit
- **authentication:** basically covered (login, weak password, unauthorized).
- **route authorization:** partially covered; 401/403 cases exist but not exhaustive by risk class.
- **object-level authorization:** still under-covered for return/exchange/transition cross-scope negatives.
- **tenant/data isolation:** partially covered in list/get style flows; mutation isolation gaps remain in tests.
- **admin/internal protection:** partially covered; no full non-admin abuse matrix.

### 8.4 Final Coverage Judgment
- **Partial Pass**
- Major critical improvement is test-backed for independent approver rule and reversal flow. However, uncovered high-risk areas (cross-scope mutation negatives, idempotency race, capability mismatch negatives, autonomous async export behavior) mean severe defects could still evade detection.

## 9. Final Notes
- Compared to the prior report, this revision clearly improves security-governance alignment and justifies upgrading from **Fail** to **Partial Pass**.
- Remaining items are concentrated and actionable; resolving the three High issues should move the project close to full pass territory in static audit.
