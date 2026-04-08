# RetailOps Static Delivery Acceptance & Architecture Audit (Fix-Check Round 3)

## 1. Verdict
- Overall conclusion: **Partial Pass**

## 2. Scope and Static Verification Boundary
- Reviewed: `repo/README.md`, core handlers under `repo/src/handlers`, RBAC guard and data-scope logic, schema/models/migrations, API and unit test assets.
- Not reviewed deeply: non-critical long-form docs and every low-risk endpoint branch.
- Intentionally not executed: service runtime, Docker, migrations, unit tests, API tests, load tests.
- Manual verification required: p95 latency under 300ms at 200 concurrent users, export throughput at 250k rows, production async scheduling behavior.

## 3. Repository / Requirement Mapping Summary
- Prompt requirements mapped to implementation across auth/lockout, role-permission bindings, API capability table, POS flows, reversal approvals, participant/team data, file governance, dataset versioning, notifications, reporting/exports, audit, observability.
- Round-3 focus checked the previously raised high-risk items: request-aware RBAC, object-scope enforcement, approval creation guard, audit hash coverage, export checksum persistence.

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability
- Conclusion: **Pass**
- Rationale: README includes startup, config, tests, security controls, contract changes, and route categories in a statically verifiable form.
- Evidence: `repo/README.md:10`, `repo/README.md:22`, `repo/README.md:45`, `repo/README.md:76`, `repo/README.md:110`.

#### 1.2 Material deviation from Prompt
- Conclusion: **Partial Pass**
- Rationale: major governance gaps were materially reduced (approval gating, scope checks on key order paths, export checksum, approval-create permission), but full 3-layer RBAC enforcement and full write-audit completeness are not yet universal.
- Evidence: `repo/src/handlers/return_handler.rs:424`, `repo/src/handlers/order_handler.rs:407`, `repo/src/handlers/order_handler.rs:523`, `repo/src/handlers/approval_handler.rs:150`, `repo/src/rbac/guard.rs:42`, `repo/src/handlers/dataset_handler.rs:31`.

### 2. Delivery Completeness

#### 2.1 Core requirement coverage
- Conclusion: **Partial Pass**
- Rationale: core APIs are broad and key critical controls improved; remaining gaps are primarily consistency of enforcement (all modules/routes) rather than total absence.
- Evidence: `repo/src/routes.rs:80`, `repo/src/routes.rs:88`, `repo/src/routes.rs:192`, `repo/src/handlers/export_handler.rs:298`, `repo/src/schema.rs:544`.

#### 2.2 End-to-end 0-to-1 deliverable quality
- Conclusion: **Pass**
- Rationale: repository has complete multi-module service structure, schema migrations, auth/permission model, and test suites/scripts.
- Evidence: `repo/src/main.rs:4`, `repo/src/models/mod.rs:1`, `repo/migrations/00000000000043_seed_approval_create_permission/up.sql:1`, `repo/API_tests/run_api_tests.sh:1`.

### 3. Engineering and Architecture Quality

#### 3.1 Structure and decomposition
- Conclusion: **Pass**
- Rationale: domain modules remain cleanly decomposed (auth, rbac, handlers, models, storage, audit, observability).
- Evidence: `repo/src/main.rs:4`, `repo/src/handlers/mod.rs:1`, `repo/src/rbac/guard.rs:1`, `repo/src/storage/mod.rs:1`.

#### 3.2 Maintainability and extensibility
- Conclusion: **Partial Pass**
- Rationale: reusable request-aware permission helper and scoped checks were added in key handlers, but enforcement style is still mixed across modules (request-aware vs non-request-aware).
- Evidence: `repo/src/handlers/order_handler.rs:20`, `repo/src/handlers/export_handler.rs:15`, `repo/src/handlers/dataset_handler.rs:31`, `repo/src/handlers/participant_handler.rs:24`.

### 4. Engineering Details and Professionalism

#### 4.1 Error handling, logging, validation, API design
- Conclusion: **Partial Pass**
- Rationale: strong validation/error handling and file governance improvements are present; audit before/after hashing is still uneven on write coverage.
- Evidence: `repo/src/errors.rs:40`, `repo/src/storage/mod.rs:132`, `repo/src/handlers/order_handler.rs:573`, `repo/src/handlers/export_handler.rs:298`, `repo/src/audit/middleware.rs:110`.

#### 4.2 Product-like service shape
- Conclusion: **Partial Pass**
- Rationale: service resembles production backend, though export completion still permits empty placeholder artifact content when none is provided.
- Evidence: `repo/src/handlers/export_handler.rs:273`, `repo/src/handlers/export_handler.rs:277`.

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business goal and constraint fit
- Conclusion: **Partial Pass**
- Rationale: implementation now better aligns with governance semantics (reversal approval flow, approval permission gate, export checksum field), but full policy enforcement consistency remains unfinished.
- Evidence: `repo/src/handlers/return_handler.rs:375`, `repo/src/handlers/approval_handler.rs:150`, `repo/src/handlers/export_handler.rs:298`, `repo/src/migrations/00000000000041_seed_api_capabilities/up.sql:6`, `repo/src/handlers/dataset_handler.rs:31`.

### 6. Aesthetics (frontend-only/full-stack)

#### 6.1 Visual and interaction design
- Conclusion: **Not Applicable**
- Rationale: backend-only deliverable.
- Evidence: `repo/README.md:3`.

## 5. Issues / Suggestions (Severity-Rated)

1) **Severity: High**  
**Title:** Layer-3 RBAC request capability enforcement is not applied consistently across all protected domains  
**Conclusion:** Partial Fail  
**Evidence:** `repo/src/rbac/guard.rs:42`, `repo/src/handlers/order_handler.rs:57`, `repo/src/handlers/export_handler.rs:28`, `repo/src/handlers/dataset_handler.rs:31`, `repo/src/handlers/approval_handler.rs:18`  
**Impact:** some modules enforce method/path capabilities while others still rely on layer 1-2 permission checks only, creating uneven authorization guarantees.  
**Minimum actionable fix:** migrate remaining protected handlers to `check_permission_for_request` (or centralized middleware) and ensure capability mappings exist for those permissions.

2) **Severity: High**  
**Title:** Full write-audit before/after hash coverage remains incomplete  
**Conclusion:** Partial Fail  
**Evidence:** `repo/src/audit/middleware.rs:102`, `repo/src/audit/middleware.rs:110`, `repo/src/audit/middleware.rs:111`, `repo/src/handlers/team_handler.rs:34`, `repo/src/handlers/dataset_handler.rs:40`  
**Impact:** many write operations can still generate audit rows without before/after hashes, weakening strict forensic traceability requirements.  
**Minimum actionable fix:** add handler-level `audit_write` with before/after snapshots for all state-changing endpoints not yet instrumented.

3) **Severity: Medium**  
**Title:** API test coverage still lacks capability-mismatch and cross-scope negative matrix  
**Conclusion:** Partial Pass  
**Evidence:** `repo/API_tests/run_api_tests.sh:546`, `repo/API_tests/run_api_tests.sh:590`, `repo/API_tests/run_api_tests.sh:597`  
**Impact:** severe authorization regressions in remaining modules could pass without detection.  
**Minimum actionable fix:** add targeted tests: permission present but capability mismatch => 403, and cross-scope object access for multiple domains.

4) **Severity: Medium**  
**Title:** Export completion permits empty artifact payloads  
**Conclusion:** Partial Pass  
**Evidence:** `repo/src/handlers/export_handler.rs:273`, `repo/src/handlers/export_handler.rs:277`, `repo/src/handlers/export_handler.rs:278`  
**Impact:** completed export records may represent placeholder artifacts rather than meaningful generated output.  
**Minimum actionable fix:** require non-empty content for completion or enforce a worker-generated artifact contract.

5) **Severity: Low**  
**Title:** README migration count appears stale  
**Conclusion:** Partial Pass  
**Evidence:** `repo/README.md:43`, `repo/migrations/00000000000043_seed_approval_create_permission/up.sql:1`  
**Impact:** minor documentation drift.  
**Minimum actionable fix:** update migration count text to current value.

## 6. Security Review Summary
- **Authentication entry points:** **Pass** — auth/login/refresh/bootstrap with password and lockout controls remain present (`repo/src/routes.rs:16`, `repo/src/auth/password.rs:8`, `repo/src/auth/lockout.rs:28`).
- **Route-level authorization:** **Partial Pass** — major routes now use request-aware permission checks (orders/exports), but not all domains have migrated.
- **Object-level authorization:** **Partial Pass** — scope checks now cover key order payment/receipt paths and participant bulk validation (`repo/src/handlers/order_handler.rs:419`, `repo/src/handlers/order_handler.rs:496`, `repo/src/handlers/participant_handler.rs:276`), but broad cross-domain consistency is still in progress.
- **Function-level authorization:** **Pass** — reversal execution now requires approved request and mapped permission gates (`repo/src/handlers/return_handler.rs:449`, `repo/src/handlers/return_handler.rs:476`).
- **Tenant/user isolation:** **Partial Pass** — improved significantly for audited high-risk routes, but not yet proven universal across all object handlers.
- **Admin/internal/debug protection:** **Pass** — metrics and admin export operations remain permission-guarded (`repo/src/observability/health.rs:45`, `repo/src/handlers/export_handler.rs:176`).

## 7. Tests and Logging Review
- **Unit tests:** **Partial Pass** — helper/path matching tests exist (`repo/src/rbac/guard.rs:154`, `repo/src/storage/mod.rs:166`, `repo/src/audit/service.rs:92`) but still limited handler-endpoint matrix.
- **API / integration tests:** **Partial Pass** — added focused sections for audit hash checks, export checksum persistence, and approval-create permission (`repo/API_tests/run_api_tests.sh:546`, `repo/API_tests/run_api_tests.sh:569`, `repo/API_tests/run_api_tests.sh:579`); capability mismatch and cross-domain 403 coverage still sparse.
- **Logging categories / observability:** **Pass** — structured logging and health/metrics endpoints are present.
- **Sensitive-data leakage risk in logs/responses:** **Partial Pass** — no direct sensitive return exposure found in sampled checks; request logging policy still generic.

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests exist in source modules and are documented (`repo/unit_tests/README.md:3`, `repo/src/rbac/guard.rs:154`).
- API integration test suite exists and includes new governance sections (`repo/API_tests/run_api_tests.sh:438`, `repo/API_tests/run_api_tests.sh:546`).
- Test commands documented in README (`repo/README.md:45`).
- Static boundary: no test execution performed.

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Reversal approval gate | `repo/API_tests/run_api_tests.sh:438` | execute-before-approval 400 and post-approval 200 | sufficient | no explicit >24h scenario | add late-reversal approval test |
| Receipt file governance (type/hash/duplicate) | `repo/API_tests/run_api_tests.sh:495` | valid upload + type rejection + duplicate hash | basically covered | no explicit >10MB test | add oversize upload test |
| Export checksum persistence | `repo/API_tests/run_api_tests.sh:569` | checks `sha256_hash` and `file_size_bytes` | basically covered | no tamper-mismatch verification | add checksum consistency check after download |
| Approval create permission | `repo/API_tests/run_api_tests.sh:579` | cashier 403, admin 202 | sufficient | none major | keep |
| Request-aware capability enforcement | none explicit in API tests | only unit path matcher tests (`repo/src/rbac/guard.rs:159`) | insufficient | no end-to-end 403 for capability mismatch | add API test with permission but non-matching capability |
| Object/data-scope isolation across domains | partial | order path improvements implied; limited explicit matrix | insufficient | no broad cross-domain out-of-scope by-ID tests | add cross-user/location matrix for orders, teams, datasets |
| Audit before/after coverage across writes | `repo/API_tests/run_api_tests.sh:546` | checks hash presence on selected resources | insufficient | no assurance for all write endpoints | add assertions for team/dataset/notification writes |

### 8.3 Security Coverage Audit
- **Authentication:** basically covered.
- **Route authorization:** basically covered for core order/export flows; insufficient global capability-mismatch testing.
- **Object-level authorization:** improved but not comprehensively tested.
- **Tenant/data isolation:** improved, still insufficient matrix coverage.
- **Admin/internal protection:** basically covered.

### 8.4 Final Coverage Judgment
- **Partial Pass**
- Covered: core reversal governance, receipt/export governance basics, approval-create permission, selected audit/hash checks.
- Remaining gaps: capability mismatch negatives and full cross-domain scope/audit coverage could still allow important regressions to escape.

## 9. Final Notes
- This round materially improved high-risk controls and moved the project from Fail to Partial Pass on static review.
- To reach Pass, prioritize full-domain request-aware RBAC migration and universal write-audit before/after hashing.
