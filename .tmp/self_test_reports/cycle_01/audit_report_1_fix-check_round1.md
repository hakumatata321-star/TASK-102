# RetailOps Static Delivery Acceptance & Architecture Audit (Fix-Check Round 1)

## 1. Verdict
- Overall conclusion: **Fail**

## 2. Scope and Static Verification Boundary
- Reviewed: `repo/README.md`, `repo/src/**/*.rs`, `repo/migrations/**`, `repo/API_tests/run_api_tests.sh`, `repo/unit_tests/*`, `repo/Dockerfile`, `repo/docker-compose.yml`, `repo/.env.example`.
- Not reviewed in depth: long-form design docs outside direct execution path.
- Intentionally not executed: app runtime, Docker, DB, migrations, unit tests, API tests.
- Manual verification required: runtime latency/p95 targets, 250k-row async export throughput, operational behavior under concurrent load.

## 3. Repository / Requirement Mapping Summary
- Prompt target verified against code: auth+lockout, RBAC/delegation, POS/order lifecycle, returns/exchanges/reversals, participant/team/attachments, notifications, datasets/versioning/lineage/field dictionary, reporting/exports, approvals, audit, observability.
- Fix-check emphasis: prior high-risk areas (reversal approvals, object/data-scope authz, 3-layer RBAC enforcement, audit before/after hashes, receipt/export file governance, static verifiability docs).

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability
- Conclusion: **Pass**
- Rationale: clear repo-level operator README now exists with startup, env, migrations, tests, API entry points, and storage/security rules.
- Evidence: `repo/README.md:10`, `repo/README.md:22`, `repo/README.md:35`, `repo/README.md:45`, `repo/README.md:56`, `repo/README.md:100`.

#### 1.2 Material deviation from Prompt
- Conclusion: **Fail**
- Rationale: core deviations remain in RBAC capability enforcement, full-surface object-level scope enforcement, complete audit before/after coverage, and receipt/export governed-file implementation.
- Evidence: `repo/src/rbac/guard.rs:88`, `repo/src/handlers/export_handler.rs:115`, `repo/src/handlers/order_handler.rs:508`, `repo/src/models/receipt.rs:14`, `repo/src/handlers/export_handler.rs:87`.

### 2. Delivery Completeness

#### 2.1 Core requirement coverage
- Conclusion: **Partial Pass**
- Rationale: reversal approval workflow has been materially improved (request then execute-after-approval), but several explicit governance requirements are still only partially implemented.
- Evidence: `repo/src/handlers/return_handler.rs:343`, `repo/src/handlers/return_handler.rs:424`, `repo/src/handlers/return_handler.rs:449`, `repo/src/handlers/return_handler.rs:470`, `repo/src/models/receipt.rs:14`.

#### 2.2 End-to-end 0-to-1 deliverable quality
- Conclusion: **Partial Pass**
- Rationale: complete multi-module service exists with schema/tests/scripts/docs; however export completion path is still marked/supported as placeholder behavior and receipt attachment remains JSON-only model.
- Evidence: `repo/src/handlers/export_handler.rs:87`, `repo/src/handlers/export_handler.rs:266`, `repo/src/handlers/order_handler.rs:482`.

### 3. Engineering and Architecture Quality

#### 3.1 Structure and decomposition
- Conclusion: **Pass**
- Rationale: modular domain decomposition remains coherent across handlers/models/rbac/auth/audit/observability/storage.
- Evidence: `repo/src/main.rs:4`, `repo/src/handlers/mod.rs:1`, `repo/src/models/mod.rs:1`, `repo/src/routes.rs:11`.

#### 3.2 Maintainability and extensibility
- Conclusion: **Partial Pass**
- Rationale: reusable scope helper and audit helper were added, but enforcement is inconsistent across endpoints and weakens maintainability of security invariants.
- Evidence: `repo/src/rbac/data_scope.rs:57`, `repo/src/handlers/participant_handler.rs:148`, `repo/src/handlers/team_handler.rs:92`, `repo/src/handlers/export_handler.rs:115`.

### 4. Engineering Details and Professionalism

#### 4.1 Error handling, logging, validation, API design
- Conclusion: **Partial Pass**
- Rationale: centralized errors/middleware/validation are present; path safety checks and approval validation improved; but several high-impact controls are still incomplete (RBAC capability matching, full audit hashes).
- Evidence: `repo/src/errors.rs:40`, `repo/src/main.rs:49`, `repo/src/storage/mod.rs:116`, `repo/src/handlers/return_handler.rs:449`, `repo/src/audit/middleware.rs:110`.

#### 4.2 Product-level shape vs demo
- Conclusion: **Partial Pass**
- Rationale: service resembles product architecture, yet some critical paths remain demonstrative placeholders.
- Evidence: `repo/src/handlers/export_handler.rs:87`, `repo/src/handlers/export_handler.rs:261`.

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business/constraint fit
- Conclusion: **Partial Pass**
- Rationale: major correction for reversal approval gate is implemented, but requirement-fit gaps remain in capability-scope enforcement, comprehensive data-scope enforcement, and receipt/export governance semantics.
- Evidence: `repo/src/handlers/return_handler.rs:375`, `repo/src/rbac/guard.rs:88`, `repo/src/handlers/order_handler.rs:490`, `repo/src/models/receipt.rs:52`.

### 6. Aesthetics (frontend-only/full-stack)

#### 6.1 Visual/interaction quality
- Conclusion: **Not Applicable**
- Rationale: backend API project only.
- Evidence: `repo/README.md:3`.

## 5. Issues / Suggestions (Severity-Rated)

1) **Severity: High**  
**Title:** RBAC layer-3 API capability enforcement is not request-scoped  
**Conclusion:** Fail  
**Evidence:** `repo/src/rbac/guard.rs:88`, `repo/src/rbac/guard.rs:90`, `repo/src/rbac/guard.rs:92`, `repo/migrations/00000000000005_create_api_capabilities/up.sql:1`  
**Impact:** system does not enforce `http_method + path_pattern` capability at runtime; permission check mostly degrades to existence/count checks.  
**Minimum actionable fix:** pass request method/path into guard (or middleware), match against capability rows, and deny on mismatch.

2) **Severity: High**  
**Title:** Object-level/data-scope authorization remains inconsistent across handlers  
**Conclusion:** Fail  
**Evidence:** `repo/src/handlers/participant_handler.rs:204`, `repo/src/handlers/participant_handler.rs:226`, `repo/src/handlers/team_handler.rs:147`, `repo/src/handlers/team_handler.rs:169`, `repo/src/handlers/order_handler.rs:215`, `repo/src/handlers/export_handler.rs:115`  
**Impact:** users with coarse permissions may access/mutate out-of-scope records via by-ID endpoints.  
**Minimum actionable fix:** apply `enforce_scope` or owner/admin checks uniformly on all object-level reads/writes/downloads and member/tag operations.

3) **Severity: High**  
**Title:** Full audit before/after hash requirement is still not met for all writes  
**Conclusion:** Fail  
**Evidence:** `repo/src/audit/middleware.rs:102`, `repo/src/audit/middleware.rs:110`, `repo/src/audit/middleware.rs:111`, `repo/src/handlers/order_handler.rs:104`, `repo/src/handlers/participant_handler.rs:48`  
**Impact:** many write operations continue logging without before/after hashes, reducing forensic/compliance traceability.  
**Minimum actionable fix:** make write handlers provide before/after payload hashes consistently (or enrich middleware with resource snapshots for covered resources).

4) **Severity: High**  
**Title:** Receipt governance still persists JSON payload instead of governed local file pointer + hash  
**Conclusion:** Fail  
**Evidence:** `repo/src/models/receipt.rs:14`, `repo/src/models/receipt.rs:53`, `repo/migrations/00000000000015_create_receipts/up.sql:5`, `repo/src/handlers/order_handler.rs:508`  
**Impact:** receipt handling does not meet file-attachment governance requirements (local disk pointer, file metadata/hash controls).  
**Minimum actionable fix:** refactor receipts to use managed file storage + DB pointer/hash metadata and enforce content-type/size constraints.

5) **Severity: Medium**  
**Title:** Export completion remains partially placeholder and may not produce a real artifact  
**Conclusion:** Partial Pass  
**Evidence:** `repo/src/handlers/export_handler.rs:261`, `repo/src/handlers/export_handler.rs:266`, `repo/src/handlers/export_handler.rs:271`, `repo/src/handlers/export_handler.rs:386`  
**Impact:** completed export record may point to a synthesized path/file state that is not generated by real export pipeline logic.  
**Minimum actionable fix:** persist actual generated artifact bytes/path from worker flow, store checksum/size from file, and avoid placeholder empty-byte writes.

6) **Severity: Medium**  
**Title:** Manual approval request creation endpoint lacks explicit permission guard  
**Conclusion:** Partial Pass  
**Evidence:** `repo/src/routes.rs:83`, `repo/src/handlers/approval_handler.rs:144`, `repo/src/handlers/approval_handler.rs:149`  
**Impact:** any authenticated caller can submit arbitrary approval requests (potential spam/abuse/noise).  
**Minimum actionable fix:** require dedicated permission (e.g., `approval.request.create`) and validate requested permission point against caller's allowed domain.

7) **Severity: Low**  
**Title:** Path safety helper does not explicitly reject drive-letter absolute paths on Windows syntax  
**Conclusion:** Partial Pass  
**Evidence:** `repo/src/storage/mod.rs:117`  
**Impact:** static path validation is less robust for cross-platform path formats.  
**Minimum actionable fix:** additionally reject patterns like `^[A-Za-z]:\\` and canonicalize against storage root.

## 6. Security Review Summary
- **Authentication entry points:** **Pass** — login/refresh/bootstrap with password policy + lockout controls remain in place (`repo/src/routes.rs:16`, `repo/src/auth/password.rs:8`, `repo/src/auth/lockout.rs:28`).
- **Route-level authorization:** **Partial Pass** — most endpoints use permission guard; one approval-creation route lacks explicit permission check (`repo/src/handlers/approval_handler.rs:149`).
- **Object-level authorization:** **Fail** — scope checks exist in some handlers but not uniformly across object operations (`repo/src/handlers/participant_handler.rs:148`, `repo/src/handlers/team_handler.rs:92`, `repo/src/handlers/order_handler.rs:215`, `repo/src/handlers/export_handler.rs:115`).
- **Function-level authorization:** **Partial Pass** — reversal execute now requires approved request (`repo/src/handlers/return_handler.rs:449`), but capability-level method/path constraints are not truly enforced (`repo/src/rbac/guard.rs:88`).
- **Tenant / user isolation:** **Partial Pass** — list filters + some object checks exist, but bypass surfaces remain in unscoped object endpoints.
- **Admin / internal / debug protection:** **Partial Pass** — metrics requires `system.health` (`repo/src/observability/health.rs:45`), health remains intentionally public (`repo/src/observability/health.rs:9`).

## 7. Tests and Logging Review
- **Unit tests:** **Partial Pass** — strong helper-level tests exist (password, crypto, storage, state machine, audit hashing), but little direct handler authz/audit lifecycle testing.
- **API / integration tests:** **Partial Pass** — new sections validate reversal approval and basic audit/path-traversal checks (`repo/API_tests/run_api_tests.sh:429`, `repo/API_tests/run_api_tests.sh:471`, `repo/API_tests/run_api_tests.sh:486`), but still lack robust scope/capability negative matrix.
- **Logging categories / observability:** **Pass** — structured logs, request metrics, health/metrics endpoints are present (`repo/src/main.rs:49`, `repo/src/observability/request_metrics.rs:8`, `repo/src/observability/health.rs:38`).
- **Sensitive-data leakage risk in logs/responses:** **Partial Pass** — response masking/no password hash exposure checks exist (`repo/API_tests/run_api_tests.sh:171`), but generic request logger still records full request line (`repo/src/main.rs:50`).

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests exist via `#[cfg(test)]` modules and shell runner: `repo/unit_tests/run_unit_tests.sh:1`.
- API integration tests exist in `repo/API_tests/run_api_tests.sh:1`.
- Test commands documented in `repo/README.md:45`.
- Static audit only; tests not executed.

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Reversal must require approval before mutation | `repo/API_tests/run_api_tests.sh:429` | execute-before-approval -> 400 (`repo/API_tests/run_api_tests.sh:453`) | basically covered | no late-reversal (>24h) explicit case | add late order fixture + assert `order.reverse_late` flow |
| Reversal idempotency | `repo/API_tests/run_api_tests.sh:465` | replay same key returns 200 (`repo/API_tests/run_api_tests.sh:467`) | basically covered | no conflicting payload same key test | add same key + different approval/order payload conflict case |
| Object-level data-scope isolation | none meaningful | N/A | missing | no cross-user/location/department 403 tests | add multi-user negative tests for participant/team/order/export by ID |
| RBAC capability scope enforcement (method+path) | none | N/A | missing | no capability mismatch 403 tests | add tests where permission exists but capability for route/method does not |
| Audit before/after hashes on writes | `repo/API_tests/run_api_tests.sh:473` | checks `after_hash` presence (`repo/API_tests/run_api_tests.sh:475`) | insufficient | no `before_hash` verification and no broad write-surface coverage | add tests on update/delete + assert both hashes where applicable |
| Export path traversal safety | `repo/API_tests/run_api_tests.sh:492` | traversal rejected 400 (`repo/API_tests/run_api_tests.sh:493`) | basically covered | no owner/non-owner download 403 test and no artifact integrity assertions | add ownership/admin boundary tests + checksum/path integrity checks |
| Password policy/lockout | login bad password case (`repo/API_tests/run_api_tests.sh:127`) + unit policy tests (`repo/src/auth/password.rs:52`) | weak password rejected (`repo/API_tests/run_api_tests.sh:175`) | insufficient | no 5-failure lockout timing test | add lockout lifecycle integration test |

### 8.3 Security Coverage Audit
- **Authentication:** basically covered, but lockout-depth tests remain insufficient.
- **Route authorization:** insufficient coverage of 403 role/capability boundaries.
- **Object-level authorization:** missing meaningful negative coverage.
- **Tenant/data isolation:** missing cross-scope negative tests.
- **Admin/internal protection:** partial coverage only.

### 8.4 Final Coverage Judgment
- **Fail**
- Covered: basic happy paths, selected approval flow, selected audit and path traversal checks.
- Uncovered high risks: capability-level authorization, object/data-scope isolation matrix, broad before/after audit integrity checks; severe defects could still pass current tests.

## 9. Final Notes
- This fix-check confirms meaningful progress on reversal approval flow and static documentation quality.
- However, unresolved high-severity governance/security gaps remain; project is not yet acceptable against the full prompt constraints.
