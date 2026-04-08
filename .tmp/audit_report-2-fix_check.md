# RetailOps Static Audit Report (Fix Check Round 2)

## 1. Verdict
- **Overall conclusion: Partial Pass**
- Significant progress is verified (async worker introduced, transition scope enforcement added, independent approver retained, expanded API tests), but there are still material requirement gaps that prevent a full Pass.

## 2. Scope and Static Verification Boundary
- **Reviewed:** `src/main.rs`, `src/export_worker.rs`, `src/handlers/{order,return,export,approval,register,dataset}_handler.rs`, `src/pos/idempotency.rs`, `src/rbac/guard.rs`, `migrations/00000000000044_seed_critical_api_capabilities/up.sql`, `API_tests/run_api_tests.sh`, `README.md`.
- **Not reviewed exhaustively:** every non-critical handler path and all historical migrations.
- **Intentionally not executed:** app runtime, Docker, migrations, tests, load/perf checks.
- **Manual verification required:** real throughput/latency behavior, background worker lifecycle under load, concurrent idempotency races in production-like conditions.

## 3. Repository / Requirement Mapping Summary
- **Prompt core target:** governed POS + analytics backend with strict RBAC/data-scope/idempotency/approval/audit controls and async export capability up to 250k rows.
- **Mapped updates:**
  - Export worker added and spawned.
  - Transition endpoint now enforces object scope and writes before/after audit.
  - Idempotency reserve/finalize API introduced and used in payment flow.
  - API tests expanded for async progression and transition scope checks.

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability
- **Conclusion: Pass**
- **Rationale:** docs are clear and now include explicit compliance notes for newly added controls.
- **Evidence:** `repo/README.md:45`, `repo/README.md:93`, `repo/README.md:134`.

#### 1.2 Material deviation from Prompt
- **Conclusion: Partial Pass**
- **Rationale:** major deviations reduced, but export scale semantics and full idempotency atomicity are still not fully aligned with prompt constraints.
- **Evidence:** `repo/src/export_worker.rs:140`, `repo/src/handlers/return_handler.rs:172`, `repo/src/handlers/return_handler.rs:549`.

### 2. Delivery Completeness

#### 2.1 Coverage of explicit core requirements
- **Conclusion: Partial Pass**
- **Rationale:** broad functional coverage exists and key previously flagged controls were improved; remaining gaps are concentrated in idempotency/completion semantics.
- **Evidence:** `repo/src/main.rs:45`, `repo/src/export_worker.rs:16`, `repo/src/handlers/order_handler.rs:337`, `repo/src/pos/idempotency.rs:44`.

#### 2.2 End-to-end deliverable vs partial demo
- **Conclusion: Partial Pass**
- **Rationale:** worker is now real in-process, but output generation currently caps data rows at 1000, conflicting with required export scale behavior.
- **Evidence:** `repo/src/export_worker.rs:140`, `repo/src/export_worker.rs:144`.

### 3. Engineering and Architecture Quality

#### 3.1 Structure and module decomposition
- **Conclusion: Pass**
- **Rationale:** clean modular structure remains, with export worker isolated in dedicated module.
- **Evidence:** `repo/src/main.rs:10`, `repo/src/export_worker.rs:1`, `repo/src/routes.rs:319`.

#### 3.2 Maintainability and extensibility
- **Conclusion: Partial Pass**
- **Rationale:** targeted improvements are maintainable, but enforcement is still mixed (atomic idempotency not applied consistently across all stock/accounting-impacting writes).
- **Evidence:** `repo/src/handlers/order_handler.rs:443`, `repo/src/handlers/return_handler.rs:54`, `repo/src/handlers/return_handler.rs:172`.

### 4. Engineering Details and Professionalism

#### 4.1 Error handling, logging, validation, API design
- **Conclusion: Partial Pass**
- **Rationale:** validation/audit quality improved; logging remains classic formatted access log rather than clearly structured JSON event logging.
- **Evidence:** `repo/src/main.rs:54`, `repo/src/main.rs:55`, `repo/src/handlers/order_handler.rs:392`.

#### 4.2 Product-level service shape
- **Conclusion: Partial Pass**
- **Rationale:** product-like backend with autonomous worker is evident; some strict compliance-level requirements remain only partially met.
- **Evidence:** `repo/src/export_worker.rs:31`, `repo/src/handlers/export_handler.rs:94`.

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business/constraint fit
- **Conclusion: Partial Pass**
- **Rationale:** understanding is strong and improved, but unresolved high-risk mismatches still exist (export row scale and idempotency scope breadth).
- **Evidence:** `repo/src/export_worker.rs:140`, `repo/src/handlers/return_handler.rs:204`, `repo/src/handlers/return_handler.rs:449`.

### 6. Aesthetics (frontend-only)
- **Conclusion: Not Applicable**
- **Rationale:** backend-only delivery.
- **Evidence:** `repo/src/main.rs:1`.

## 5. Issues / Suggestions (Severity-Rated)

### High

1) **Async export implementation does not satisfy required 250,000-row scale behavior**
- **Conclusion:** Fail
- **Evidence:** `repo/src/export_worker.rs:140`, `repo/src/export_worker.rs:144`
- **Impact:** worker hard-caps generated rows at 1000 (`estimated.min(1000)`), so prompt-required high-volume export behavior is not materially implemented.
- **Minimum actionable fix:** generate full dataset up to requested volume (including 250k), stream/chunk writes to disk, and keep progress updates while processing.

2) **Atomic idempotency is not consistently applied to all stock/accounting-impacting write paths**
- **Conclusion:** Fail
- **Evidence:** `repo/src/handlers/order_handler.rs:443`, `repo/src/handlers/return_handler.rs:54`, `repo/src/handlers/return_handler.rs:172`, `repo/src/handlers/return_handler.rs:449`, `repo/src/handlers/return_handler.rs:549`
- **Impact:** some critical mutation paths (return/exchange/reversal) still use check+store pattern rather than reserve+finalize, leaving race windows for duplicate side effects.
- **Minimum actionable fix:** adopt `reserve_idempotency_key` + `finalize_idempotency` in all impacting mutation transactions, not only payments.

### Medium

3) **Layer-3 RBAC request-aware enforcement remains inconsistent on approvals/register close**
- **Conclusion:** Partial Fail
- **Evidence:** `repo/src/handlers/approval_handler.rs:53`, `repo/src/handlers/register_handler.rs:24`, `repo/src/rbac/guard.rs:42`
- **Impact:** three-layer RBAC is present but not uniformly request-aware across all critical endpoints.
- **Minimum actionable fix:** use `check_permission_for_request` where `HttpRequest` is available for remaining critical endpoints.

4) **Structured logging requirement still only partially satisfied**
- **Conclusion:** Partial Fail
- **Evidence:** `repo/src/main.rs:54`, `repo/src/main.rs:55`
- **Impact:** logs are parseable but not strongly structured JSON as required by strict observability interpretation.
- **Minimum actionable fix:** adopt JSON logger format with stable fields (request_id, user_id, path, status, latency, error code).

## 6. Security Review Summary
- **authentication entry points — Pass**
  - Auth + lockout controls remain in place.
  - Evidence: `repo/src/handlers/auth_handler.rs:46`, `repo/src/auth/lockout.rs:28`.

- **route-level authorization — Partial Pass**
  - Request-aware checks are used in many critical routes, but not all.
  - Evidence: `repo/src/handlers/order_handler.rs:330`, `repo/src/handlers/export_handler.rs:28`, `repo/src/handlers/approval_handler.rs:53`.

- **object-level authorization — Pass (for previously flagged POS mutation gap)**
  - Transition/return/exchange/reversal paths enforce scope against target order.
  - Evidence: `repo/src/handlers/order_handler.rs:338`, `repo/src/handlers/return_handler.rs:64`, `repo/src/handlers/return_handler.rs:213`, `repo/src/handlers/return_handler.rs:484`.

- **function-level authorization — Pass**
  - Independent approver prohibition remains enforced.
  - Evidence: `repo/src/handlers/approval_handler.rs:65`, `repo/src/handlers/approval_handler.rs:129`.

- **tenant/user data isolation — Partial Pass**
  - Scope checks are improved on major POS mutations; full-project isolation remains broad and not exhaustively proven here.
  - Evidence: `repo/src/handlers/order_handler.rs:338`, `repo/src/handlers/participant_handler.rs:70`.

- **admin/internal/debug protection — Partial Pass**
  - Admin export operations are permission-gated; full adversarial matrix not statically proven.
  - Evidence: `repo/src/handlers/export_handler.rs:201`, `repo/src/handlers/audit_handler.rs:17`.

## 7. Tests and Logging Review
- **Unit tests — Partial Pass**
  - Unit-level coverage exists (including RBAC path matching), but concurrency/idempotency race behavior remains under-covered statically.
  - Evidence: `repo/src/rbac/guard.rs:154`, `repo/unit_tests/README.md:7`.

- **API / integration tests — Partial Pass (improved)**
  - Added checks for async worker progression and cross-scope transition denial.
  - Cross-scope return/exchange and capability-mismatch negative tests are still not clearly present.
  - Evidence: `repo/API_tests/run_api_tests.sh:615`, `repo/API_tests/run_api_tests.sh:655`, `repo/API_tests/run_api_tests.sh:629`.

- **Logging categories / observability — Partial Pass**
  - Health/metrics middleware present; log formatting remains non-JSON.
  - Evidence: `repo/src/observability/health.rs:38`, `repo/src/observability/metrics.rs:8`, `repo/src/main.rs:54`.

- **Sensitive-data leakage risk in logs/responses — Partial Pass**
  - Masking/non-leak tests exist; runtime operational logging content still requires manual governance checks.
  - Evidence: `repo/src/models/user.rs:58`, `repo/API_tests/run_api_tests.sh:504`.

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests and API integration tests both exist and are documented.
- Evidence: `repo/unit_tests/run_unit_tests.sh:31`, `repo/API_tests/run_api_tests.sh:1`, `repo/README.md:47`.

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Independent approver | `repo/API_tests/run_api_tests.sh:465` | self-approve returns 403 | sufficient | Reject self-test absent | Add self-reject negative case |
| Reversal approval lifecycle | `repo/API_tests/run_api_tests.sh:450`, `repo/API_tests/run_api_tests.sh:461`, `repo/API_tests/run_api_tests.sh:481` | pre-approval blocked, post-approval succeeds | basically covered | Cross-scope reversal abuse case missing | Add out-of-scope reversal execution 403 |
| Async export worker progression | `repo/API_tests/run_api_tests.sh:620`, `repo/API_tests/run_api_tests.sh:629` | after wait, status not queued | basically covered | Does not validate 250k-scale behavior | Add large-estimate async job test with progress and completion assertions |
| Idempotency duplicate safety | `repo/API_tests/run_api_tests.sh:637`, `repo/API_tests/run_api_tests.sh:641` | replay value unchanged | insufficient | Not a concurrent race test; only replay check | Add parallel submission race test on same key |
| Transition scope enforcement | `repo/API_tests/run_api_tests.sh:655`, `repo/API_tests/run_api_tests.sh:656` | cross-scope transition blocked 403 | sufficient | Return/exchange scope negatives missing | Add cross-scope return + exchange tests |
| Audit before/after hashes on transitions | `repo/API_tests/run_api_tests.sh:664`, `repo/API_tests/run_api_tests.sh:666`, `repo/API_tests/run_api_tests.sh:667` | both hash fields asserted | basically covered | Not all critical write endpoints validated | Add representative write matrix hash assertions |
| Layer-3 capability mismatch | No direct negative test located | N/A | missing | possible regressions undetected | Add 403 test for method/path mismatch capability |

### 8.3 Security Coverage Audit
- **authentication:** basically covered.
- **route authorization:** partially covered; improved but not exhaustive.
- **object-level authorization:** materially improved for transition/reversal; return/exchange negative tests still thin.
- **tenant/data isolation:** partially covered in tests.
- **admin/internal protection:** partially covered; deeper abuse matrix missing.

### 8.4 Final Coverage Judgment
- **Partial Pass**
- Coverage quality improved and validates several repaired paths, but major risk points (full idempotency race safety, 250k export behavior, capability-mismatch enforcement tests) are not yet sufficiently proven by static test assets.

## 9. Final Notes
- This round is a clear improvement over Round 1 and keeps the project close to Pass.
- A full Pass in strict static acceptance likely requires closing the two High issues (250k export semantics and full atomic idempotency coverage across all impacting writes).
