# RetailOps Static Delivery Acceptance & Architecture Audit (Fix-Check Round 2)

## 1. Verdict
- Overall conclusion: **Fail**

## 2. Scope and Static Verification Boundary
- Reviewed: `repo/README.md`, `repo/src/**/*.rs`, `repo/migrations/**`, `repo/API_tests/run_api_tests.sh`, `repo/unit_tests/README.md`, `repo/Dockerfile`, `repo/docker-compose.yml`, `repo/.env.example`.
- Not reviewed deeply: non-execution design narratives outside the active code path.
- Intentionally not executed: runtime server, tests, Docker, DB migrations, load/performance checks.
- Manual verification required: runtime latency/throughput claims, concurrency behavior at 200 users, asynchronous export processing under large payloads.

## 3. Repository / Requirement Mapping Summary
- Prompt core mapped to implementation: auth+lockout, RBAC/delegation, POS and reversal approvals, participant/team data, attachments, datasets/versioning, notifications, reporting/exports, audits, health/metrics.
- Round-2 fix-check focus: previously raised high-severity areas (approval bypass, object/data-scope, RBAC layer-3 capability enforcement, audit before/after hash completeness, receipt/export file governance).

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability
- Conclusion: **Pass**
- Rationale: clear startup/config/test/API/security docs exist at repository level and are statically actionable.
- Evidence: `repo/README.md:10`, `repo/README.md:22`, `repo/README.md:45`, `repo/README.md:56`, `repo/README.md:76`, `repo/README.md:100`.

#### 1.2 Material deviation from Prompt
- Conclusion: **Fail**
- Rationale: key governance constraints still not fully enforced: request-level API capability RBAC is implemented but not wired into handlers/middleware, object-level scope remains inconsistent on POS/bulk paths, audit before/after hashes are not comprehensive, and export artifact fingerprint is not persisted in job storage.
- Evidence: `repo/src/rbac/guard.rs:42`, `repo/src/handlers/order_handler.rs:46`, `repo/src/handlers/order_handler.rs:316`, `repo/src/handlers/order_handler.rs:392`, `repo/src/audit/middleware.rs:110`, `repo/src/migrations/00000000000036_create_export_jobs/up.sql:11`.

### 2. Delivery Completeness

#### 2.1 Core requirement coverage
- Conclusion: **Partial Pass**
- Rationale: reversal approval gate and receipt multipart governance were materially improved, but full prompt completeness still misses strict layer-3 runtime enforcement and complete write-audit/hash and scope coverage.
- Evidence: `repo/src/handlers/return_handler.rs:343`, `repo/src/handlers/return_handler.rs:424`, `repo/src/handlers/order_handler.rs:487`, `repo/src/rbac/guard.rs:42`, `repo/src/audit/middleware.rs:102`.

#### 2.2 End-to-end 0-to-1 deliverable
- Conclusion: **Partial Pass**
- Rationale: service structure is end-to-end and production-shaped, but some critical controls still depend on partial implementation and missing verification coverage.
- Evidence: `repo/src/main.rs:45`, `repo/src/routes.rs:11`, `repo/API_tests/run_api_tests.sh:438`, `repo/API_tests/run_api_tests.sh:529`.

### 3. Engineering and Architecture Quality

#### 3.1 Structure and decomposition
- Conclusion: **Pass**
- Rationale: domain decomposition is coherent (auth/rbac/handlers/models/storage/audit/observability), and implementation is not monolithic.
- Evidence: `repo/src/main.rs:4`, `repo/src/handlers/mod.rs:1`, `repo/src/models/mod.rs:1`, `repo/src/routes.rs:3`.

#### 3.2 Maintainability and extensibility
- Conclusion: **Partial Pass**
- Rationale: reusable helpers were added (`check_permission_for_request`, scope helpers, audit helper), but enforcement remains uneven across endpoints, increasing drift risk.
- Evidence: `repo/src/rbac/guard.rs:42`, `repo/src/rbac/data_scope.rs:57`, `repo/src/handlers/order_handler.rs:222`, `repo/src/handlers/order_handler.rs:471`, `repo/src/handlers/participant_handler.rs:273`.

### 4. Engineering Details and Professionalism

#### 4.1 Error handling, logging, validation, API design
- Conclusion: **Partial Pass**
- Rationale: robust error mapping, validation, and structured middleware are present; path/file governance improved for receipts/exports; however capability RBAC and full write-audit integrity are not complete.
- Evidence: `repo/src/errors.rs:40`, `repo/src/storage/mod.rs:16`, `repo/src/handlers/order_handler.rs:538`, `repo/src/handlers/export_handler.rs:272`, `repo/src/audit/middleware.rs:110`.

#### 4.2 Product-level organization vs demo
- Conclusion: **Partial Pass**
- Rationale: service mostly resembles a real product, but export completion still allows empty placeholder artifacts and async worker behavior is not fully implemented/proven.
- Evidence: `repo/src/handlers/export_handler.rs:87`, `repo/src/handlers/export_handler.rs:265`, `repo/src/handlers/export_handler.rs:266`.

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business goal and constraints fit
- Conclusion: **Partial Pass**
- Rationale: major governance intent is better reflected (reversal approval, owner/admin export access, receipt local storage), yet remaining high-risk gaps prevent full fit against strict prompt constraints.
- Evidence: `repo/src/handlers/return_handler.rs:375`, `repo/src/handlers/export_handler.rs:121`, `repo/src/handlers/export_handler.rs:377`, `repo/src/rbac/guard.rs:42`, `repo/src/migrations/00000000000036_create_export_jobs/up.sql:11`.

### 6. Aesthetics (frontend-only/full-stack)

#### 6.1 Visual and interaction quality
- Conclusion: **Not Applicable**
- Rationale: backend API only.
- Evidence: `repo/README.md:3`.

## 5. Issues / Suggestions (Severity-Rated)

1) **Severity: High**  
**Title:** Request-level API capability RBAC exists but is not enforced in runtime paths  
**Conclusion:** Fail  
**Evidence:** `repo/src/rbac/guard.rs:42`, `repo/src/rbac/guard.rs:52`, `repo/src/handlers/order_handler.rs:46`, `repo/src/handlers/export_handler.rs:21`, `repo/src/routes.rs:91`  
**Impact:** role+permission checks can pass without method/path capability matching, weakening the required 3-layer RBAC model.  
**Minimum actionable fix:** replace relevant `check_permission(...)` calls with request-aware `check_permission_for_request(...)` (or equivalent middleware) and pass actual method/path context.

2) **Severity: High**  
**Title:** API capability data likely unenforced by default due missing seeded mappings  
**Conclusion:** Fail  
**Evidence:** `repo/migrations/00000000000005_create_api_capabilities/up.sql:1`, `repo/migrations/00000000000018_seed_pos_permissions/up.sql:2`, `repo/migrations/00000000000037_seed_reporting_permissions/up.sql:1`  
**Impact:** even with request-aware guard available, no capability rows means layer-3 can be skipped system-wide, violating prompt-level enforcement intent.  
**Minimum actionable fix:** seed `api_capabilities` for protected endpoints and enforce deny-by-default when capabilities are expected.

3) **Severity: High**  
**Title:** Object-level/data-scope authorization still inconsistent on POS and bulk operations  
**Conclusion:** Fail  
**Evidence:** `repo/src/handlers/order_handler.rs:392`, `repo/src/handlers/order_handler.rs:471`, `repo/src/handlers/order_handler.rs:497`, `repo/src/handlers/participant_handler.rs:273`, `repo/src/handlers/participant_handler.rs:295`  
**Impact:** callers with broad permission codes may perform out-of-scope by-ID payment/receipt/list operations or bulk actions across unauthorized entities.  
**Minimum actionable fix:** enforce scope checks for order payment/receipt/list-payments paths and per-entity scope validation in bulk participant operations.

4) **Severity: High**  
**Title:** Full write audit integrity (before/after hashes) remains incomplete  
**Conclusion:** Fail  
**Evidence:** `repo/src/audit/middleware.rs:110`, `repo/src/audit/middleware.rs:111`, `repo/src/handlers/team_handler.rs:34`, `repo/src/handlers/team_handler.rs:133`, `repo/src/handlers/export_handler.rs:311`  
**Impact:** many writes continue generating audit records without before/after hashes, so compliance requirement for complete write traceability is not met.  
**Minimum actionable fix:** instrument all state-changing handlers with `audit_write` before/after payloads (or equivalent comprehensive mechanism) and avoid null hash writes for covered resources.

5) **Severity: High**  
**Title:** Export artifact fingerprint is computed but not persisted with export metadata  
**Conclusion:** Fail  
**Evidence:** `repo/src/handlers/export_handler.rs:272`, `repo/src/handlers/export_handler.rs:295`, `repo/migrations/00000000000036_create_export_jobs/up.sql:11`, `repo/src/models/export_job.rs:33`  
**Impact:** export tamper/duplicate detection lacks first-class persisted checksum at artifact record level, contrary to file governance requirement for exports.  
**Minimum actionable fix:** add export checksum column (e.g., `sha256_hash`) and persist/return it on completion/download metadata.

6) **Severity: Medium**  
**Title:** Approval request creation endpoint still lacks explicit permission gate  
**Conclusion:** Partial Pass  
**Evidence:** `repo/src/routes.rs:83`, `repo/src/handlers/approval_handler.rs:149`, `repo/src/handlers/approval_handler.rs:157`  
**Impact:** any authenticated user may submit arbitrary approval requests, increasing abuse/noise risk.  
**Minimum actionable fix:** require dedicated `approval.request.create` permission and validate allowed target permission points.

7) **Severity: Medium**  
**Title:** README migration count is stale versus repository migrations  
**Conclusion:** Partial Pass  
**Evidence:** `repo/README.md:43`, `repo/migrations/00000000000040_add_receipt_file_columns/up.sql:1`  
**Impact:** minor static-verifiability drift for operators.  
**Minimum actionable fix:** update README migration count and keep docs synced with migration history.

## 6. Security Review Summary
- **Authentication entry points:** **Pass** — auth endpoints and password/lockout controls remain present (`repo/src/routes.rs:16`, `repo/src/auth/password.rs:8`, `repo/src/auth/lockout.rs:28`).
- **Route-level authorization:** **Partial Pass** — permissions are checked broadly, but capability-layer request matching is not wired in and one approval creation path lacks explicit gate (`repo/src/rbac/guard.rs:42`, `repo/src/handlers/approval_handler.rs:149`).
- **Object-level authorization:** **Partial Pass** — improved in participant/team/export owner access, but POS payment/receipt/list-payments and participant bulk scope validation still incomplete (`repo/src/handlers/team_handler.rs:173`, `repo/src/handlers/export_handler.rs:121`, `repo/src/handlers/order_handler.rs:392`, `repo/src/handlers/participant_handler.rs:273`).
- **Function-level authorization:** **Partial Pass** — reversal now requires approved request before financial mutation (`repo/src/handlers/return_handler.rs:449`, `repo/src/handlers/return_handler.rs:476`), but capability enforcement remains incomplete.
- **Tenant/user isolation:** **Partial Pass** — many endpoints use `enforce_scope`, but not uniformly on all object/bulk operations.
- **Admin/internal/debug protection:** **Partial Pass** — metrics protected (`repo/src/observability/health.rs:45`), health public by design (`repo/src/observability/health.rs:9`), approval creation caveat remains.

## 7. Tests and Logging Review
- **Unit tests:** **Partial Pass** — helper-level tests exist and include storage/audit/path matching, but limited endpoint-level authorization/audit completeness tests (`repo/src/rbac/guard.rs:154`, `repo/src/storage/mod.rs:257`, `repo/src/audit/service.rs:92`).
- **API/integration tests:** **Partial Pass** — added tests for reversal approval, receipt governance, export artifact completion; still missing capability-mismatch and cross-scope negative matrix (`repo/API_tests/run_api_tests.sh:438`, `repo/API_tests/run_api_tests.sh:495`, `repo/API_tests/run_api_tests.sh:529`).
- **Logging/observability:** **Pass** — structured request logs, request metrics, health/metrics endpoints remain available (`repo/src/main.rs:49`, `repo/src/observability/request_metrics.rs:8`, `repo/src/observability/health.rs:38`).
- **Sensitive-data leakage risk in logs/responses:** **Partial Pass** — no direct evidence of password hash response leakage; generic request-line logging still exists and redaction policy is not explicit (`repo/src/main.rs:50`).

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests exist as inline Rust tests (`repo/src/rbac/guard.rs:154`, `repo/src/storage/mod.rs:166`, `repo/src/audit/service.rs:92`).
- API integration tests exist in Bash/curl (`repo/API_tests/run_api_tests.sh:1`).
- Test commands are documented (`repo/README.md:45`).
- Static-only assessment: tests were not executed.

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Reversal must require approval before mutation | `repo/API_tests/run_api_tests.sh:438` | execute-before-approval is 400 (`repo/API_tests/run_api_tests.sh:462`) | sufficient | no explicit >24h late reversal case | add late-order reversal approval test |
| Reversal idempotency | `repo/API_tests/run_api_tests.sh:473` | replay same key returns 200 (`repo/API_tests/run_api_tests.sh:476`) | basically covered | no conflicting payload check | add same-key different payload conflict test |
| Receipt file governance (type/hash/duplicate) | `repo/API_tests/run_api_tests.sh:495` | valid upload + disallowed type + duplicate hash (`repo/API_tests/run_api_tests.sh:504`, `repo/API_tests/run_api_tests.sh:515`, `repo/API_tests/run_api_tests.sh:524`) | basically covered | no explicit oversized file test | add >10MB upload rejection test |
| Export artifact completion path | `repo/API_tests/run_api_tests.sh:529` | complete with base64 and owner download (`repo/API_tests/run_api_tests.sh:537`, `repo/API_tests/run_api_tests.sh:542`) | basically covered | no non-owner/admin negative test; no checksum persistence assertion | add owner-vs-admin download 403 tests + checksum field assertions |
| RBAC method/path capability enforcement | unit path helper only (`repo/src/rbac/guard.rs:159`) | helper-level path matching tests | insufficient | no endpoint test proving capability mismatch causes 403 | add integration tests with capability map mismatch by method/path |
| Object-level/data-scope isolation | limited | basic auth and not-found checks only | missing | no cross-scope 403 matrix on object/bulk paths | add multi-user location/department boundary tests |
| Full before/after audit hash on writes | audit query assertions (`repo/API_tests/run_api_tests.sh:482`) | checks for `after_hash` only (`repo/API_tests/run_api_tests.sh:484`) | insufficient | before_hash and broad write-surface coverage missing | add update/delete checks asserting before+after where applicable |

### 8.3 Security Coverage Audit
- **Authentication:** basically covered.
- **Route authorization:** insufficient for capability-layer enforcement.
- **Object-level authorization:** insufficient; critical cross-scope negative paths are not covered.
- **Tenant/data isolation:** insufficient.
- **Admin/internal protection:** partial (owner download positive only; missing non-owner negative checks).

### 8.4 Final Coverage Judgment
- **Fail**
- Covered: key happy paths, reversal approval gate, receipt file flow basics, export completion happy path.
- Not covered enough: capability-level RBAC mismatch behavior, object-level scope boundary negatives, and comprehensive before/after audit hashing across write operations; severe defects could still pass.

## 9. Final Notes
- This round shows clear improvements versus prior state (notably reversal approval and receipt/export input hardening).
- Remaining unresolved high-severity control gaps keep the static acceptance verdict at Fail.
