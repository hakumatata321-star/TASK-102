# RetailOps Static Audit Report (Self-Test Cycle 02)

## 1. Verdict
- **Overall conclusion: Fail**
- Rationale: the repository is substantial and statically verifiable, but multiple **Blocker/High** gaps remain against the prompt: critical approval second-confirmation is not enforced, object-level authorization is missing on several POS mutation flows, full write-audit before/after hash coverage is incomplete, and async export behavior is simulated rather than implemented.

## 2. Scope and Static Verification Boundary
- **Reviewed:** architecture/docs/config (`README.md`, `Dockerfile`, `docker-compose.yml`, `.env.example`), routing/entrypoints (`src/main.rs`, `src/routes.rs`), security/auth/RBAC/data-scope modules, core handlers (auth, POS, returns/reversals, register close, datasets, notifications, reports/exports, audit), storage/encryption, migrations, unit/API test artifacts.
- **Not reviewed exhaustively:** generated build outputs under `target/`.
- **Intentionally not executed:** application runtime, Docker, migrations execution, tests, load/perf checks, external/manual workflows.
- **Manual verification required:** p95 latency under 300ms @ 200 concurrency, real async export throughput to 250k rows, runtime correctness of all complex flows.

## 3. Repository / Requirement Mapping Summary
- **Prompt core goal:** offline-first RetailOps governance backend (Actix + Diesel + Postgres) covering auth, 3-layer RBAC + data-scope/delegation, POS lifecycle + idempotency + reconciliation approvals, participant/attachment management, notifications, analytics/reporting/exports, dataset version lineage/rollback, encryption/masking/auditability/observability.
- **Mapped implementation areas:**
  - Entry + infra: `src/main.rs`, `src/routes.rs`, `Dockerfile`, `docker-compose.yml`.
  - Security/RBAC: `src/auth/*`, `src/rbac/*`, `src/security/csrf.rs`, migrations 03/07/08/11/18/37/41/43.
  - POS + approvals/idempotency: `src/handlers/order_handler.rs`, `src/handlers/return_handler.rs`, `src/handlers/register_handler.rs`, `src/pos/*`.
  - Data governance: `src/handlers/dataset_handler.rs`, migrations 25-29.
  - Notifications/reporting/exports: `src/handlers/notification_handler.rs`, `src/handlers/report_handler.rs`, `src/handlers/export_handler.rs`.
  - Audit/observability/tests: `src/audit/*`, `src/observability/*`, `unit_tests/*`, `API_tests/run_api_tests.sh`.

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability
- **Conclusion: Pass**
- **Rationale:** startup/config/test instructions exist and are mostly consistent with code/manifests.
- **Evidence:** `repo/README.md:10`, `repo/README.md:22`, `repo/README.md:45`, `repo/Dockerfile:41`, `repo/docker-compose.yml:19`, `repo/src/main.rs:30`.

#### 1.2 Material deviation from prompt
- **Conclusion: Fail**
- **Rationale:** several core prompt constraints are weakened or unmet (approval second-confirmation, broad 3-layer API-scope enforcement, async export execution, full hashed audit coverage).
- **Evidence:** `repo/src/handlers/approval_handler.rs:82`, `repo/src/handlers/return_handler.rs:50`, `repo/src/handlers/export_handler.rs:95`, `repo/src/audit/middleware.rs:111`, `repo/src/rbac/guard.rs:11`, `repo/migrations/00000000000041_seed_api_capabilities/up.sql:1`.

### 2. Delivery Completeness

#### 2.1 Coverage of explicit core requirements
- **Conclusion: Partial Pass**
- **Rationale:** broad module coverage exists (auth, RBAC, POS, participant/team, datasets/versioning/rollback, notifications, reports/exports, audit, metrics), but critical requirement-level gaps remain.
- **Evidence:** `repo/src/routes.rs:13`, `repo/src/handlers/auth_handler.rs:46`, `repo/src/handlers/order_handler.rs:320`, `repo/src/handlers/dataset_handler.rs:173`, `repo/src/handlers/notification_handler.rs:131`, `repo/src/handlers/report_handler.rs:149`, `repo/src/handlers/export_handler.rs:21`.

#### 2.2 End-to-end deliverable vs partial demo
- **Conclusion: Partial Pass**
- **Rationale:** project is multi-module and production-shaped, but export execution is explicitly simulated in-process rather than a real async worker flow.
- **Evidence:** `repo/src/handlers/export_handler.rs:95`, `repo/src/handlers/export_handler.rs:97`, `repo/README.md:127`.

### 3. Engineering and Architecture Quality

#### 3.1 Structure and module decomposition
- **Conclusion: Pass**
- **Rationale:** clear modular decomposition by domain and cross-cutting concerns.
- **Evidence:** `repo/src/main.rs:4`, `repo/src/routes.rs:3`, `repo/src/handlers/mod.rs:1`, `repo/src/models/mod.rs:1`.

#### 3.2 Maintainability and extensibility
- **Conclusion: Partial Pass**
- **Rationale:** generally maintainable structure, but several security/business-rule checks are inconsistent across handlers, increasing long-term defect risk.
- **Evidence:** `repo/src/handlers/order_handler.rs:236`, `repo/src/handlers/return_handler.rs:50`, `repo/src/rbac/guard.rs:11`.

### 4. Engineering Details and Professionalism

#### 4.1 Error handling, logging, validation, API design
- **Conclusion: Partial Pass**
- **Rationale:** error mapping/validation is present; observability endpoints exist; however logs are standard text access logs (not structured JSON), and several critical write paths miss before/after hash audit evidence.
- **Evidence:** `repo/src/errors.rs:40`, `repo/src/main.rs:49`, `repo/src/observability/health.rs:38`, `repo/src/audit/middleware.rs:102`, `repo/src/handlers/order_handler.rs:220`.

#### 4.2 Product-level delivery vs demo-level
- **Conclusion: Partial Pass**
- **Rationale:** service resembles a real backend, but key prompt-critical controls still rely on simplified patterns (simulated async export; incomplete enforcement consistency).
- **Evidence:** `repo/src/handlers/export_handler.rs:95`, `repo/src/handlers/report_handler.rs:166`.

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business goal and implicit constraints fit
- **Conclusion: Fail**
- **Rationale:** implementation understands domain breadth, but mismatches core semantics in governance controls: second-confirmation, comprehensive API-scope enforcement, true async export execution, and complete hashed audit trails.
- **Evidence:** `repo/src/handlers/approval_handler.rs:73`, `repo/migrations/00000000000018_seed_pos_permissions/up.sql:40`, `repo/src/rbac/guard.rs:11`, `repo/src/handlers/export_handler.rs:95`, `repo/src/audit/middleware.rs:111`.

### 6. Aesthetics (frontend-only)
- **Conclusion: Not Applicable**
- **Rationale:** backend-only repository; no frontend UI scope.
- **Evidence:** `repo/src/main.rs:1`, `repo/Cargo.toml:7`.

## 5. Issues / Suggestions (Severity-Rated)

### Blocker

1) **Second-confirmation requirement is not enforced for critical approvals**
- **Conclusion:** Fail
- **Evidence:** `repo/src/handlers/approval_handler.rs:73`, `repo/src/handlers/approval_handler.rs:82`, `repo/src/handlers/register_handler.rs:191`, `repo/migrations/00000000000018_seed_pos_permissions/up.sql:40`
- **Impact:** requester can approve/complete their own critical actions; contradicts required manager second-confirmation for >$20 variance and late reversals.
- **Minimum actionable fix:** enforce approver != requester in approval decision logic; require policy-level second actor semantics (at least one independent approver) for variance and late reversal flows.

2) **Object-level authorization missing on critical POS mutation endpoints**
- **Conclusion:** Fail
- **Evidence:** `repo/src/handlers/return_handler.rs:50`, `repo/src/handlers/return_handler.rs:196`, `repo/src/handlers/return_handler.rs:373`, `repo/src/handlers/order_handler.rs:330`
- **Impact:** users with broad permission code may mutate orders outside their data scope (returns/exchanges/reversals/transitions).
- **Minimum actionable fix:** use returned `PermissionContext` and enforce owner/department/location checks before each mutation, consistent with `order_handler` scoped operations.

### High

3) **Three-layer RBAC (role→permission→API scope) is only partially enforced**
- **Conclusion:** Fail
- **Evidence:** `repo/src/rbac/guard.rs:11`, `repo/src/handlers/participant_handler.rs:61`, `repo/src/handlers/report_handler.rs:26`, `repo/migrations/00000000000041_seed_api_capabilities/up.sql:1`
- **Impact:** many endpoints enforce only layers 1-2, weakening the prompt-mandated API capability scope boundary.
- **Minimum actionable fix:** move all protected handlers to request-aware checks (`check_permission_for_request`) and seed capability mappings for full protected surface.

4) **Full write-audit before/after hash coverage is incomplete**
- **Conclusion:** Fail
- **Evidence:** `repo/src/audit/middleware.rs:111`, `repo/src/audit/middleware.rs:112`, `repo/src/handlers/order_handler.rs:220`, `repo/src/handlers/notification_handler.rs:131`
- **Impact:** many writes may only have metadata audit entries without before/after hashes, violating full forensic traceability requirement.
- **Minimum actionable fix:** require explicit `audit_write` with canonical before/after state for every write endpoint (or enhance middleware to capture hashable state reliably).

5) **Export generation is not implemented as true asynchronous processing**
- **Conclusion:** Fail
- **Evidence:** `repo/src/handlers/export_handler.rs:94`, `repo/src/handlers/export_handler.rs:95`, `repo/src/handlers/export_handler.rs:97`
- **Impact:** prompt asks async exports up to 250,000 rows with progress; current flow is state simulation plus admin-driven completion payload.
- **Minimum actionable fix:** implement worker/job execution loop (same node, local only), transition queued→running→completed/failed autonomously, and persist progress updates during generation.

6) **Idempotency control is non-atomic for some stock/accounting-impacting writes**
- **Conclusion:** Fail
- **Evidence:** `repo/src/pos/idempotency.rs:12`, `repo/src/handlers/return_handler.rs:53`, `repo/src/handlers/return_handler.rs:168`, `repo/src/handlers/return_handler.rs:325`
- **Impact:** concurrent duplicate submissions can pass pre-check before key storage and perform duplicate side effects.
- **Minimum actionable fix:** enforce atomic idempotency reservation (insert-first with unique key in same transaction before mutation) for all impacting operations.

7) **Report dimensions/filters are mostly stored/echoed but not materially applied in KPI queries**
- **Conclusion:** Partial Fail
- **Evidence:** `repo/src/handlers/report_handler.rs:166`, `repo/src/handlers/report_handler.rs:174`, `repo/src/handlers/report_handler.rs:182`
- **Impact:** configurable analytics behavior from prompt is only partially realized; report outputs may ignore configured dimensions/filters.
- **Minimum actionable fix:** build query builder logic using definition/runtime dimensions and filter clauses per KPI type; validate supported dimensions explicitly.

### Medium

8) **Structured logging requirement is only partially met**
- **Conclusion:** Partial Fail
- **Evidence:** `repo/src/main.rs:49`, `repo/src/main.rs:50`, `repo/src/main.rs:25`
- **Impact:** troubleshooting/audit ingestion is harder vs required structured logs.
- **Minimum actionable fix:** switch to structured JSON logging format with stable keys (request_id/user_id/path/status/duration/error_code).

## 6. Security Review Summary

- **authentication entry points — Pass**
  - Local username/password with policy and lockout are present.
  - Evidence: `repo/src/handlers/auth_handler.rs:46`, `repo/src/auth/password.rs:8`, `repo/src/auth/lockout.rs:28`.

- **route-level authorization — Partial Pass**
  - Most protected endpoints require `AuthenticatedUser` + permission checks.
  - API-scope (layer 3) is not universal.
  - Evidence: `repo/src/routes.rs:23`, `repo/src/auth/middleware.rs:22`, `repo/src/rbac/guard.rs:11`.

- **object-level authorization — Fail**
  - Several critical order mutation handlers do not enforce data-scope/object ownership.
  - Evidence: `repo/src/handlers/return_handler.rs:50`, `repo/src/handlers/return_handler.rs:373`, `repo/src/handlers/order_handler.rs:330`.

- **function-level authorization — Partial Pass**
  - Permission checks exist broadly, including approval-gated permission semantics.
  - Second-confirmation independence not enforced.
  - Evidence: `repo/src/rbac/guard.rs:19`, `repo/src/handlers/approval_handler.rs:73`.

- **tenant / user data isolation — Partial Pass**
  - Many list/get paths apply data-scope filters; some critical flows bypass them.
  - Evidence: `repo/src/handlers/participant_handler.rs:70`, `repo/src/handlers/order_handler.rs:142`, `repo/src/handlers/return_handler.rs:57`.

- **admin / internal / debug protection — Partial Pass**
  - Metrics, audit, admin notification/export routes are permission-protected.
  - No separate debug endpoints found.
  - Evidence: `repo/src/observability/health.rs:45`, `repo/src/routes.rs:324`, `repo/src/routes.rs:346`.

## 7. Tests and Logging Review

- **Unit tests — Partial Pass**
  - Utility/domain logic has unit tests (password, masking, crypto, state machine, storage, metrics).
  - Core handler authz/object-scope/approval invariants have little direct unit coverage.
  - Evidence: `repo/unit_tests/README.md:7`, `repo/src/pos/state_machine.rs:33`, `repo/src/storage/mod.rs:166`.

- **API / integration tests — Partial Pass**
  - Broad shell-based coverage exists across many endpoints and several negative cases.
  - Critical segregation tests (self-approval prohibition, object-level scope abuse, delegation boundaries) are not meaningfully covered.
  - Evidence: `repo/API_tests/run_api_tests.sh:113`, `repo/API_tests/run_api_tests.sh:438`, `repo/API_tests/run_api_tests.sh:590`.

- **Logging categories / observability — Partial Pass**
  - Health and metrics endpoints exist; request/error counters collected.
  - Logging is text access-style, not clearly structured event logs.
  - Evidence: `repo/src/observability/health.rs:38`, `repo/src/observability/metrics.rs:8`, `repo/src/main.rs:49`.

- **Sensitive-data leakage risk in logs / responses — Partial Pass**
  - Password hash not returned by user APIs; gov_id display masked.
  - Internal error details are logged server-side (`log::error!`), requiring operational log hygiene review.
  - Evidence: `repo/src/models/user.rs:58`, `repo/src/errors.rs:81`, `repo/API_tests/run_api_tests.sh:171`.

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests exist in `#[cfg(test)]` Rust modules; API integration tests are shell/curl scripts.
- Framework/tooling: Rust `cargo test` + bash/curl assertions.
- Test entry points documented and scripted.
- Evidence: `repo/unit_tests/README.md:3`, `repo/unit_tests/run_unit_tests.sh:31`, `repo/API_tests/run_api_tests.sh:1`, `repo/README.md:47`.

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Password policy + auth basics | `repo/API_tests/run_api_tests.sh:123`, `repo/API_tests/run_api_tests.sh:174`; `repo/src/auth/password.rs:57` | login success/fail, weak password 400 | basically covered | No explicit lockout duration/assertion | Add API test for 5 failed attempts + lockout window behavior |
| 401 unauthenticated access | `repo/API_tests/run_api_tests.sh:134` | `/roles` without token => 401 | sufficient | Limited to one route | Add 401 checks on at least one critical mutation endpoint |
| 403 unauthorized permission | `repo/API_tests/run_api_tests.sh:592` | cashier denied `/approvals` create | basically covered | Does not cover object-level forbidden | Add cross-scope order reversal/transition forbidden test |
| POS idempotency duplicate payment | `repo/API_tests/run_api_tests.sh:212` | same key replay returns same ID | sufficient | No concurrent duplicate race test | Add parallel duplicate submission test |
| Reversal approval gating | `repo/API_tests/run_api_tests.sh:450`, `repo/API_tests/run_api_tests.sh:461`, `repo/API_tests/run_api_tests.sh:469` | execute before approval rejected; after approval succeeds | basically covered | No independent-approver test; self-approval currently allowed | Add test asserting requester cannot approve own critical request |
| Dataset version lineage + rollback request | `repo/API_tests/run_api_tests.sh:293`, `repo/API_tests/run_api_tests.sh:318` | version lineage and rollback request accepted | basically covered | Missing execute_rollback approval lifecycle test | Add rollback approve+execute negative/positive cases |
| File governance (type/size/hash) | `repo/API_tests/run_api_tests.sh:499`, `repo/API_tests/run_api_tests.sh:510`, `repo/API_tests/run_api_tests.sh:519`; `repo/src/storage/mod.rs:241` | valid upload, disallowed type, duplicate hash | sufficient | No >10MB API boundary test | Add oversized multipart upload rejection test |
| Export lifecycle + checksum | `repo/API_tests/run_api_tests.sh:532`, `repo/API_tests/run_api_tests.sh:536`, `repo/API_tests/run_api_tests.sh:572` | complete+download+sha256 persistence | basically covered | Does not verify autonomous async worker behavior | Add test asserting queued jobs progress without manual `/complete` |
| Audit hash presence | `repo/API_tests/run_api_tests.sh:482`, `repo/API_tests/run_api_tests.sh:549` | asserts `after_hash`/`before_hash` fields in filtered outputs | insufficient | Tests do not prove all write endpoints hash before/after | Add matrix test over representative write endpoints verifying hash fields non-null |
| Data-scope/object isolation | (No clear targeted tests found) | N/A | missing | Critical scope bypass risk in return/reversal/transition not tested | Add multi-user scope isolation suite with cross-location/department fixtures |

### 8.3 Security Coverage Audit
- **authentication:** basically covered (login/refresh/weak password), but lockout timing behavior not proven.
- **route authorization:** basically covered for some 401/403 paths, not comprehensive across high-risk mutations.
- **object-level authorization:** missing meaningful tests; severe defects could pass current suite.
- **tenant/data isolation:** insufficient; limited checks do not validate cross-scope data mutation prevention.
- **admin/internal protection:** basically covered for metrics/audit/admin export paths under admin token use, but no adversarial non-admin matrix.

### 8.4 Final Coverage Judgment
- **Partial Pass**
- Major happy-paths and many endpoint contracts are covered, but uncovered high-risk areas (object-level authorization, independent approval actor, full audit-hash coverage, async export execution realism, concurrency idempotency races) mean severe defects could still remain undetected while tests pass.

## 9. Final Notes
- This is a static-only assessment; runtime/performance claims are not asserted.
- Highest-priority repair focus should be: (1) second-confirmation enforcement, (2) object-level scope checks on all critical POS mutations, (3) universal layer-3 RBAC enforcement, (4) comprehensive before/after hash audit coverage, and (5) true async export execution path.
