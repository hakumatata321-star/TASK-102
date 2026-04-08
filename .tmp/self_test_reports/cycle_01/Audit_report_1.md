# RetailOps Static Delivery Acceptance & Architecture Audit (Self-Test)

## 1. Verdict
- Overall conclusion: **Fail**

## 2. Scope and Static Verification Boundary
- Reviewed: Rust source under `repo/src`, migrations under `repo/migrations`, configuration/deployment files (`repo/Cargo.toml`, `repo/Dockerfile`, `repo/docker-compose.yml`, `repo/.env.example`), and test assets (`repo/run_tests.sh`, `repo/API_tests/run_api_tests.sh`, `repo/unit_tests/README.md`).
- Not reviewed in depth: very large narrative docs outside code execution path except where needed for doc/code consistency (`docs/design.md`, `docs/api-spec.md`).
- Intentionally not executed: application runtime, Docker/Compose, migrations, unit tests, API tests, external services.
- Manual verification required for runtime claims (latency p95 at 200 users, async export throughput at 250k rows, operational behavior under concurrency/failures).

## 3. Repository / Requirement Mapping Summary
- Prompt goal mapped: offline RetailOps governance API across auth, RBAC, POS, participant management, notifications, datasets/versioning, analytics/reporting, file handling, approvals, audit, observability.
- Main implementation areas found: route surface (`repo/src/routes.rs:11`), auth/JWT/lockout (`repo/src/handlers/auth_handler.rs:46`, `repo/src/auth/password.rs:8`, `repo/src/auth/lockout.rs:20`), RBAC checks (`repo/src/rbac/guard.rs:18`), POS/returns/register (`repo/src/handlers/order_handler.rs:37`, `repo/src/handlers/return_handler.rs:41`, `repo/src/handlers/register_handler.rs:18`), datasets/versioning (`repo/src/handlers/dataset_handler.rs:22`), notifications (`repo/src/handlers/notification_handler.rs:18`), reports/exports (`repo/src/handlers/report_handler.rs:17`, `repo/src/handlers/export_handler.rs:15`), audit/metrics (`repo/src/audit/middleware.rs:8`, `repo/src/observability/health.rs:9`).

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability
- Conclusion: **Partial Pass**
- Rationale: project has substantial static artifacts (Dockerfile, Compose, env example, API/design docs, test scripts), but there is no concise repository-root operator README and several design statements are not aligned with code behavior.
- Evidence: `repo/Dockerfile:1`, `repo/docker-compose.yml:1`, `repo/.env.example:1`, `repo/run_tests.sh:1`, `docs/design.md:1477`, `repo/src/main.rs:45`, `repo/src/db.rs:9`.
- Manual verification note: runtime startup and ops instructions are partly inferable, but reviewer must reconcile doc/code mismatches manually.

#### 1.2 Material deviation from Prompt
- Conclusion: **Fail**
- Rationale: several core governance constraints are weakened or bypassed: reversal approval bypass, missing 3-layer RBAC enforcement, incomplete file-governed receipts/exports, and incomplete audit before/after hashing.
- Evidence: `repo/src/handlers/return_handler.rs:367`, `repo/src/handlers/order_handler.rs:326`, `repo/src/rbac/guard.rs:31`, `repo/src/handlers/order_handler.rs:501`, `repo/migrations/00000000000015_create_receipts/up.sql:5`, `repo/src/audit/middleware.rs:109`.

### 2. Delivery Completeness

#### 2.1 Core requirement coverage
- Conclusion: **Fail**
- Rationale: broad endpoint surface exists, but key explicit requirements are incomplete: approval gating for reversals is bypassed; receipts are JSON blobs not governed file attachments; async export generation is simulated and not actually produced; object-level/data-scope isolation is inconsistent.
- Evidence: `repo/src/routes.rs:88`, `repo/src/handlers/return_handler.rs:376`, `repo/src/handlers/order_handler.rs:332`, `repo/src/models/receipt.rs:14`, `repo/src/handlers/export_handler.rs:87`, `repo/src/handlers/participant_handler.rs:136`.

#### 2.2 End-to-end 0-to-1 deliverable quality
- Conclusion: **Partial Pass**
- Rationale: codebase is multi-module with migrations and scripts (not a toy snippet), but parts of required behavior are placeholder/simulated, especially export execution workflow.
- Evidence: `repo/src/models/mod.rs:1`, `repo/migrations/00000000000036_create_export_jobs/up.sql:1`, `repo/src/handlers/export_handler.rs:87`.

### 3. Engineering and Architecture Quality

#### 3.1 Structure and decomposition
- Conclusion: **Pass**
- Rationale: module split is generally coherent by domain (auth, handlers, models, rbac, pos, observability, storage, migrations) and avoids single-file pile-up.
- Evidence: `repo/src/main.rs:4`, `repo/src/handlers/mod.rs:1`, `repo/src/models/mod.rs:1`, `repo/src/routes.rs:3`.

#### 3.2 Maintainability and extensibility
- Conclusion: **Partial Pass**
- Rationale: maintainable model/handler layering exists, but security-critical logic is repeated and inconsistently applied (scope checks present on list endpoints but omitted on many object endpoints).
- Evidence: `repo/src/handlers/participant_handler.rs:67`, `repo/src/handlers/participant_handler.rs:136`, `repo/src/handlers/team_handler.rs:52`, `repo/src/handlers/team_handler.rs:85`, `repo/src/handlers/register_handler.rs:140`, `repo/src/handlers/register_handler.rs:179`.

### 4. Engineering Details and Professionalism

#### 4.1 Error handling, logging, validation, API design
- Conclusion: **Partial Pass**
- Rationale: centralized error type and HTTP mapping exist, structured request logging and metrics middleware exist, and many request DTOs validate inputs; however, critical authorization/audit details are incomplete.
- Evidence: `repo/src/errors.rs:40`, `repo/src/main.rs:49`, `repo/src/observability/request_metrics.rs:8`, `repo/src/auth/password.rs:8`, `repo/src/audit/middleware.rs:109`.

#### 4.2 Product-level organization vs demo
- Conclusion: **Partial Pass**
- Rationale: repository shape resembles a service product, but export execution is explicitly simulated and some governance controls remain non-production-grade.
- Evidence: `repo/src/handlers/export_handler.rs:87`, `repo/src/handlers/export_handler.rs:90`.

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Semantic fit to business goal and constraints
- Conclusion: **Fail**
- Rationale: implementation captures many domain surfaces, but misses key governance semantics (approval enforcement consistency, layered RBAC enforcement semantics, file-governed receipts/exports, complete audit before/after coverage).
- Evidence: `repo/src/rbac/guard.rs:31`, `repo/src/schema.rs:104`, `repo/src/handlers/return_handler.rs:371`, `repo/src/models/receipt.rs:53`, `repo/src/audit/middleware.rs:110`.

### 6. Aesthetics (frontend-only/full-stack)

#### 6.1 Visual/interaction quality
- Conclusion: **Not Applicable**
- Rationale: backend-only API project; no frontend/UI deliverable in scope.
- Evidence: `docs/design.md:39`.

## 5. Issues / Suggestions (Severity-Rated)

### Blocker / High

1) **Severity: Blocker**  
**Title:** Critical reversal approval workflow is bypassable  
**Conclusion:** Fail  
**Evidence:** `repo/src/handlers/return_handler.rs:367`, `repo/src/handlers/return_handler.rs:376`, `repo/src/handlers/order_handler.rs:326`, `repo/src/handlers/order_handler.rs:332`  
**Impact:** Reversals (including >24h) can be executed directly without enforced approval gate, violating mandatory critical-action approval requirements and creating financial control risk.  
**Minimum actionable fix:** For reversal actions, create `approval_requests` first and only execute ledger/order mutation after approved status (pattern similar to dataset rollback request/execute flow).

2) **Severity: High**  
**Title:** Object-level/data-scope authorization is inconsistently enforced  
**Conclusion:** Fail  
**Evidence:** `repo/src/handlers/participant_handler.rs:67`, `repo/src/handlers/participant_handler.rs:136`, `repo/src/handlers/team_handler.rs:52`, `repo/src/handlers/team_handler.rs:85`, `repo/src/handlers/register_handler.rs:179`, `repo/src/handlers/export_handler.rs:115`  
**Impact:** Users with broad read/update permissions may access or modify records outside location/department/individual scope by direct ID access.  
**Minimum actionable fix:** Apply `PermissionContext` scope checks on all object endpoints (`get/update/delete/download/confirm`) and enforce owner-or-admin rules for export job retrieval/download.

3) **Severity: High**  
**Title:** RBAC three-layer enforcement is incomplete (role → permission → API capability)  
**Conclusion:** Fail  
**Evidence:** `repo/src/rbac/guard.rs:31`, `repo/src/rbac/guard.rs:41`, `repo/src/schema.rs:104`, `repo/src/schema.rs:115`  
**Impact:** API capability and menu scope bindings exist as data but are not enforcement gates; governance model can drift from configured policy.  
**Minimum actionable fix:** Extend guard/middleware to resolve request method/path against `api_capabilities` for the permission point; keep menu scopes as explicit capability map for clients.

4) **Severity: High**  
**Title:** Audit trail does not capture before/after state hashes  
**Conclusion:** Fail  
**Evidence:** `repo/src/audit/middleware.rs:109`, `repo/src/audit/middleware.rs:110`, `repo/src/audit/middleware.rs:111`, `repo/src/audit/service.rs:30`  
**Impact:** Write audit records lack required before/after hash traceability, weakening forensic and compliance guarantees.  
**Minimum actionable fix:** Capture pre-mutation and post-mutation canonical snapshots (or deterministic hash payloads) per resource mutation and pass into `audit::service::record`.

5) **Severity: High**  
**Title:** Receipt/export file-governance requirements are only partially implemented  
**Conclusion:** Fail  
**Evidence:** `repo/src/models/receipt.rs:14`, `repo/migrations/00000000000015_create_receipts/up.sql:5`, `repo/src/handlers/order_handler.rs:501`, `repo/src/handlers/export_handler.rs:87`, `repo/src/handlers/export_handler.rs:385`  
**Impact:** Receipts are stored as JSON in DB instead of governed local files with hash pointers; export generation is not implemented end-to-end and depends on manual completion path input.  
**Minimum actionable fix:** Implement actual receipt/export file creation to local disk with validated type/size and SHA-256 persisted in DB; remove trust-on-input file paths for completion.

### Medium / Low

6) **Severity: Medium**  
**Title:** Documentation-to-code mismatch on operations details  
**Conclusion:** Partial Pass  
**Evidence:** `docs/design.md:1442`, `repo/src/db.rs:9`, `docs/design.md:1483`, `repo/src/main.rs:45`, `docs/design.md:1385`, `repo/src/config.rs:20`  
**Impact:** Reviewers/operators may follow inaccurate assumptions (pool sizing, background tasks, encryption key source).  
**Minimum actionable fix:** Align design docs with implemented behavior or implement documented startup/background behavior.

7) **Severity: Medium**  
**Title:** Security test coverage gaps for authorization boundaries  
**Conclusion:** Fail  
**Evidence:** `repo/API_tests/run_api_tests.sh:134`, `repo/API_tests/run_api_tests.sh:409`, `repo/API_tests/run_api_tests.sh:415`  
**Impact:** Severe authz defects (scope bypass, object-level leaks) could pass current test suite undetected.  
**Minimum actionable fix:** Add negative tests for 403 across role/scope/object boundaries and ownership checks for ID-based access.

## 6. Security Review Summary

- **Authentication entry points:** **Partial Pass** — login/refresh/bootstrap exist and password/lockout policies are implemented (`repo/src/routes.rs:16`, `repo/src/handlers/auth_handler.rs:46`, `repo/src/auth/password.rs:8`, `repo/src/auth/lockout.rs:28`); static review cannot confirm brute-force/rate-limit runtime controls.
- **Route-level authorization:** **Partial Pass** — most non-auth handlers require `AuthenticatedUser` and permission checks (`repo/src/handlers/order_handler.rs:39`, `repo/src/handlers/order_handler.rs:46`), but `create_approval_request` lacks explicit permission gate (`repo/src/handlers/approval_handler.rs:144`).
- **Object-level authorization:** **Fail** — multiple by-ID handlers omit data-scope ownership checks (`repo/src/handlers/participant_handler.rs:136`, `repo/src/handlers/team_handler.rs:85`, `repo/src/handlers/register_handler.rs:179`, `repo/src/handlers/export_handler.rs:115`).
- **Function-level authorization:** **Partial Pass** — permission-point checks are pervasive, but critical-action approval semantics are bypassed in reversal flows (`repo/src/handlers/return_handler.rs:371`, `repo/src/handlers/return_handler.rs:376`).
- **Tenant / user data isolation:** **Fail** — list endpoints often scope-filter, while object endpoints frequently do not, enabling possible cross-scope access by ID (`repo/src/handlers/participant_handler.rs:67`, `repo/src/handlers/participant_handler.rs:140`).
- **Admin / internal / debug endpoint protection:** **Partial Pass** — metrics endpoint is permission-protected (`repo/src/observability/health.rs:45`), health is intentionally public (`repo/src/observability/health.rs:9`), but export/admin visibility still has object-level caveats (`repo/src/handlers/export_handler.rs:115`).

## 7. Tests and Logging Review

- **Unit tests:** **Partial Pass** — unit tests exist for crypto/password/state machine/storage/metrics/audit hash (`repo/src/auth/password.rs:52`, `repo/src/pos/state_machine.rs:33`, `repo/src/storage/mod.rs:114`), but little coverage for handlers/RBAC/approval workflows.
- **API / integration tests:** **Partial Pass** — shell-based integration script covers many happy paths and basic errors (`repo/API_tests/run_api_tests.sh:105`), but limited negative authorization/scope cases.
- **Logging categories / observability:** **Pass** — structured request logger + request metrics + health/metrics endpoints present (`repo/src/main.rs:49`, `repo/src/observability/request_metrics.rs:8`, `repo/src/observability/health.rs:38`).
- **Sensitive-data leakage risk in logs / responses:** **Partial Pass** — password hashes are not returned and gov-id masking exists (`repo/src/models/user.rs:58`, `repo/API_tests/run_api_tests.sh:171`), but default logger includes full request line and no explicit redaction policy for sensitive query/body values (`repo/src/main.rs:50`).

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests exist as inline `#[cfg(test)]` modules in 7 source files (`repo/src/auth/password.rs:52`, `repo/src/crypto/aes.rs:44`, `repo/src/storage/mod.rs:114`).
- API integration tests exist as Bash+curl script (`repo/API_tests/run_api_tests.sh:1`).
- Test runners are provided (`repo/run_tests.sh:1`, `repo/unit_tests/run_unit_tests.sh:1`).
- Test command documentation exists in `repo/unit_tests/README.md:3` and scripts; static audit did not execute them.

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Password policy (12+ upper/lower/digit) | `repo/src/auth/password.rs:57` | `validate_password(...)` assertions (`repo/src/auth/password.rs:63`) | sufficient | None obvious | Keep + add handler-level test for create_user/bootstrap invalid payloads |
| Account lockout semantics | API script login bad password only `repo/API_tests/run_api_tests.sh:127` | single 401 check (`repo/API_tests/run_api_tests.sh:128`) | insufficient | No 5-failure lock and 15-minute lockout behavior test | Add integration test loop: 5 failures then locked response and unlock-window behavior |
| POS state machine transitions | `repo/src/pos/state_machine.rs:38` | valid/invalid transition assertions (`repo/src/pos/state_machine.rs:60`) | basically covered | Handler-level transition+permission coupling not covered | Add API tests for forbidden transition attempts by role and late-reversal approval path |
| Idempotency duplicate write handling | API payment replay `repo/API_tests/run_api_tests.sh:216` | same ID replay check (`repo/API_tests/run_api_tests.sh:220`) | basically covered | No test for conflicting payload with same key / cross-resource key misuse | Add conflict tests for same key+different payload and different endpoint reuse |
| Approval workflow for critical actions | Dataset rollback request test `repo/API_tests/run_api_tests.sh:309` | expects 202 with approval_request_id (`repo/API_tests/run_api_tests.sh:311`) | insufficient | No reversal/bulk-export approval execution tests; reversal bypass not caught | Add tests asserting reversal returns pending approval and cannot execute until approved |
| Object-level authorization/data scope | only missing-auth and 404 checks (`repo/API_tests/run_api_tests.sh:134`, `repo/API_tests/run_api_tests.sh:415`) | 401/404 assertions | missing | No 403 tests for cross-location/cross-user object IDs | Add multi-user fixture tests for participant/team/register/export object access control |
| File governance (type/size/hash) | storage unit tests (`repo/src/storage/mod.rs:121`) | allowed/disallowed type, size limit, SHA check (`repo/src/storage/mod.rs:189`) | basically covered | No API test for receipt/export file lifecycle constraints | Add API tests for receipt attachment as file, export file hash/size metadata and duplicate detection |
| Audit integrity (who/when/what/before-after hash) | hash function unit tests (`repo/src/audit/service.rs:55`) | deterministic SHA checks (`repo/src/audit/service.rs:79`) | insufficient | No tests that write operations persist before/after hashes | Add integration tests asserting non-null before_hash/after_hash for representative writes |

### 8.3 Security Coverage Audit
- **Authentication:** basically covered (login success/failure, refresh, missing token) but lockout depth insufficient (`repo/API_tests/run_api_tests.sh:123`, `repo/API_tests/run_api_tests.sh:130`, `repo/API_tests/run_api_tests.sh:134`).
- **Route authorization:** insufficient; tests check 401 and one CSRF case, but little 403 role-permission matrix coverage (`repo/API_tests/run_api_tests.sh:409`).
- **Object-level authorization:** missing; no tests exercise out-of-scope object access by ID.
- **Tenant / data isolation:** missing; no cross-location/department isolation tests.
- **Admin / internal protection:** insufficient; metrics endpoint tested with authenticated token only, not privilege boundary differentiation (`repo/API_tests/run_api_tests.sh:397`).

### 8.4 Final Coverage Judgment
- **Fail**
- Major risks covered: some happy-path API functionality, basic input errors, core unit helper logic.
- Major uncovered risks: authorization boundary integrity, object/data-scope isolation, approval gate enforcement on reversals, and audit before/after traceability. Current tests could pass while severe security/compliance defects remain.

## 9. Final Notes
- This is a static-only audit; no runtime success is asserted.
- Highest-priority remediation order: (1) approval bypass, (2) object-level/scope enforcement, (3) RBAC 3-layer enforcement, (4) audit before/after hashing, (5) receipt/export governed file pipeline.
