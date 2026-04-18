# Test Coverage Audit

## Project Type Detection
- Declared project type: **backend** (`repo/README.md:1`)
- Inference not required because declaration exists.

## Backend Endpoint Inventory
- Source of truth: `src/routes.rs:11` (`configure` function under `/api/v1` scope).
- Total unique endpoints (METHOD + fully resolved PATH): **129**

- `POST /api/v1/auth/login`
- `POST /api/v1/auth/refresh`
- `POST /api/v1/auth/bootstrap`
- `GET /api/v1/users`
- `POST /api/v1/users`
- `GET /api/v1/users/{id}`
- `GET /api/v1/roles`
- `POST /api/v1/roles`
- `GET /api/v1/roles/{id}`
- `PUT /api/v1/roles/{id}`
- `DELETE /api/v1/roles/{id}`
- `POST /api/v1/roles/{role_id}/permissions`
- `DELETE /api/v1/roles/{role_id}/permissions/{perm_id}`
- `GET /api/v1/permissions`
- `POST /api/v1/permissions`
- `GET /api/v1/permissions/{id}`
- `PUT /api/v1/permissions/{id}`
- `DELETE /api/v1/permissions/{id}`
- `GET /api/v1/api-capabilities`
- `POST /api/v1/api-capabilities`
- `GET /api/v1/api-capabilities/{id}`
- `PUT /api/v1/api-capabilities/{id}`
- `DELETE /api/v1/api-capabilities/{id}`
- `GET /api/v1/menu-scopes`
- `POST /api/v1/menu-scopes`
- `GET /api/v1/menu-scopes/{id}`
- `PUT /api/v1/menu-scopes/{id}`
- `DELETE /api/v1/menu-scopes/{id}`
- `GET /api/v1/delegations`
- `POST /api/v1/delegations`
- `POST /api/v1/delegations/{id}/revoke`
- `GET /api/v1/approvals`
- `POST /api/v1/approvals`
- `GET /api/v1/approvals/{id}`
- `POST /api/v1/approvals/{id}/approve`
- `POST /api/v1/approvals/{id}/reject`
- `POST /api/v1/orders`
- `GET /api/v1/orders`
- `GET /api/v1/orders/{id}`
- `PUT /api/v1/orders/{id}`
- `POST /api/v1/orders/{id}/transition`
- `POST /api/v1/orders/{id}/payments`
- `GET /api/v1/orders/{id}/payments`
- `POST /api/v1/orders/{id}/receipts`
- `POST /api/v1/orders/{id}/returns`
- `POST /api/v1/orders/{id}/exchanges`
- `POST /api/v1/orders/{id}/reversals`
- `POST /api/v1/orders/{id}/reversals/execute`
- `POST /api/v1/registers/close`
- `GET /api/v1/registers/closings`
- `GET /api/v1/registers/closings/{id}`
- `POST /api/v1/registers/closings/{id}/confirm`
- `POST /api/v1/participants`
- `GET /api/v1/participants`
- `POST /api/v1/participants/bulk/tag`
- `POST /api/v1/participants/bulk/deactivate`
- `GET /api/v1/participants/{id}`
- `PUT /api/v1/participants/{id}`
- `DELETE /api/v1/participants/{id}`
- `GET /api/v1/participants/{id}/tags`
- `PUT /api/v1/participants/{id}/tags`
- `POST /api/v1/participants/{id}/attachments`
- `GET /api/v1/participants/{id}/attachments`
- `GET /api/v1/participants/{id}/attachments/{attachment_id}`
- `DELETE /api/v1/participants/{id}/attachments/{attachment_id}`
- `POST /api/v1/teams`
- `GET /api/v1/teams`
- `GET /api/v1/teams/{id}`
- `PUT /api/v1/teams/{id}`
- `DELETE /api/v1/teams/{id}`
- `GET /api/v1/teams/{id}/members`
- `POST /api/v1/teams/{id}/members`
- `DELETE /api/v1/teams/{id}/members/{participant_id}`
- `POST /api/v1/datasets`
- `GET /api/v1/datasets`
- `GET /api/v1/datasets/{id}`
- `PUT /api/v1/datasets/{id}`
- `DELETE /api/v1/datasets/{id}`
- `POST /api/v1/datasets/{id}/versions`
- `GET /api/v1/datasets/{id}/versions`
- `GET /api/v1/datasets/{id}/versions/{version_id}`
- `GET /api/v1/datasets/{id}/versions/{version_id}/lineage`
- `GET /api/v1/datasets/{id}/versions/{version_id}/fields`
- `POST /api/v1/datasets/{id}/versions/{version_id}/fields`
- `PUT /api/v1/datasets/{id}/versions/{version_id}/fields/{field_id}`
- `DELETE /api/v1/datasets/{id}/versions/{version_id}/fields/{field_id}`
- `POST /api/v1/datasets/{id}/rollback`
- `POST /api/v1/datasets/{id}/rollback/execute`
- `POST /api/v1/notification-templates`
- `GET /api/v1/notification-templates`
- `GET /api/v1/notification-templates/{id}`
- `PUT /api/v1/notification-templates/{id}`
- `DELETE /api/v1/notification-templates/{id}`
- `POST /api/v1/notifications/send`
- `POST /api/v1/notifications/send-direct`
- `POST /api/v1/notifications/broadcast`
- `GET /api/v1/notifications/inbox`
- `GET /api/v1/notifications/inbox/unread-count`
- `POST /api/v1/notifications/inbox/read-all`
- `GET /api/v1/notifications/inbox/{id}`
- `POST /api/v1/notifications/inbox/{id}/read`
- `GET /api/v1/notifications/admin`
- `GET /api/v1/notifications/admin/{id}/delivery-logs`
- `POST /api/v1/notifications/admin/{id}/retry`
- `GET /api/v1/reports/kpi-types`
- `POST /api/v1/reports`
- `GET /api/v1/reports`
- `GET /api/v1/reports/{id}`
- `PUT /api/v1/reports/{id}`
- `DELETE /api/v1/reports/{id}`
- `POST /api/v1/reports/{id}/run`
- `POST /api/v1/scheduled-reports`
- `GET /api/v1/scheduled-reports`
- `GET /api/v1/scheduled-reports/{id}`
- `PUT /api/v1/scheduled-reports/{id}`
- `DELETE /api/v1/scheduled-reports/{id}`
- `POST /api/v1/exports`
- `GET /api/v1/exports`
- `GET /api/v1/exports/admin`
- `GET /api/v1/exports/{id}`
- `PUT /api/v1/exports/{id}/progress`
- `POST /api/v1/exports/{id}/complete`
- `POST /api/v1/exports/{id}/fail`
- `POST /api/v1/exports/{id}/cancel`
- `GET /api/v1/exports/{id}/download`
- `GET /api/v1/audit`
- `GET /api/v1/audit/{id}`
- `GET /api/v1/health`
- `GET /api/v1/metrics`

## API Test Mapping Table
| Endpoint | Covered | Test Type | Test Files | Evidence |
|---|---|---|---|---|
| `POST /api/v1/auth/login` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `auth_handler::login` (`src/routes.rs:17`); `http_post_noauth` at `API_tests/run_api_tests.sh:123` |
| `POST /api/v1/auth/refresh` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `auth_handler::refresh` (`src/routes.rs:18`); `http_post_noauth` at `API_tests/run_api_tests.sh:130` |
| `POST /api/v1/auth/bootstrap` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `auth_handler::bootstrap` (`src/routes.rs:19`); `http_post_noauth` at `API_tests/run_api_tests.sh:115` |
| `GET /api/v1/users` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `user_handler::list_users` (`src/routes.rs:24`); `http_get` at `API_tests/run_api_tests.sh:177` |
| `POST /api/v1/users` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `user_handler::create_user` (`src/routes.rs:25`); `http_post` at `API_tests/run_api_tests.sh:168` |
| `GET /api/v1/users/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `user_handler::get_user` (`src/routes.rs:26`); `http_get` at `API_tests/run_api_tests.sh:894` |
| `GET /api/v1/roles` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `role_handler::list` (`src/routes.rs:31`); `http_get` at `API_tests/run_api_tests.sh:141` |
| `POST /api/v1/roles` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `role_handler::create` (`src/routes.rs:32`); `http_post` at `API_tests/run_api_tests.sh:150` |
| `GET /api/v1/roles/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `role_handler::get` (`src/routes.rs:33`); `http_get` at `API_tests/run_api_tests.sh:903` |
| `PUT /api/v1/roles/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `role_handler::update` (`src/routes.rs:34`); `http_put` at `API_tests/run_api_tests.sh:157` |
| `DELETE /api/v1/roles/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `role_handler::delete` (`src/routes.rs:35`); `http_delete` at `API_tests/run_api_tests.sh:161` |
| `POST /api/v1/roles/{role_id}/permissions` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `permission_handler::bind_to_role` (`src/routes.rs:36`); `http_post` at `API_tests/run_api_tests.sh:935` |
| `DELETE /api/v1/roles/{role_id}/permissions/{perm_id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `permission_handler::unbind_from_role` (`src/routes.rs:40`); `http_delete` at `API_tests/run_api_tests.sh:940` |
| `GET /api/v1/permissions` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `permission_handler::list` (`src/routes.rs:48`); `http_get` at `API_tests/run_api_tests.sh:146` |
| `POST /api/v1/permissions` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `permission_handler::create` (`src/routes.rs:49`); `http_post` at `API_tests/run_api_tests.sh:915` |
| `GET /api/v1/permissions/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `permission_handler::get` (`src/routes.rs:50`); `http_get` at `API_tests/run_api_tests.sh:921` |
| `PUT /api/v1/permissions/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `permission_handler::update` (`src/routes.rs:51`); `http_put` at `API_tests/run_api_tests.sh:926` |
| `DELETE /api/v1/permissions/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `permission_handler::delete` (`src/routes.rs:52`); `http_delete` at `API_tests/run_api_tests.sh:948` |
| `GET /api/v1/api-capabilities` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `api_scope_handler::list` (`src/routes.rs:57`); `http_get` at `API_tests/run_api_tests.sh:956` |
| `POST /api/v1/api-capabilities` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `api_scope_handler::create` (`src/routes.rs:58`); `http_post` at `API_tests/run_api_tests.sh:961` |
| `GET /api/v1/api-capabilities/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `api_scope_handler::get` (`src/routes.rs:59`); `http_get` at `API_tests/run_api_tests.sh:967` |
| `PUT /api/v1/api-capabilities/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `api_scope_handler::update` (`src/routes.rs:60`); `http_put` at `API_tests/run_api_tests.sh:972` |
| `DELETE /api/v1/api-capabilities/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `api_scope_handler::delete` (`src/routes.rs:61`); `http_delete` at `API_tests/run_api_tests.sh:977` |
| `GET /api/v1/menu-scopes` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `menu_scope_handler::list` (`src/routes.rs:66`); `http_get` at `API_tests/run_api_tests.sh:985` |
| `POST /api/v1/menu-scopes` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `menu_scope_handler::create` (`src/routes.rs:67`); `http_post` at `API_tests/run_api_tests.sh:989` |
| `GET /api/v1/menu-scopes/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `menu_scope_handler::get` (`src/routes.rs:68`); `http_get` at `API_tests/run_api_tests.sh:995` |
| `PUT /api/v1/menu-scopes/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `menu_scope_handler::update` (`src/routes.rs:69`); `http_put` at `API_tests/run_api_tests.sh:1000` |
| `DELETE /api/v1/menu-scopes/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `menu_scope_handler::delete` (`src/routes.rs:70`); `http_delete` at `API_tests/run_api_tests.sh:1005` |
| `GET /api/v1/delegations` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `delegation_handler::list` (`src/routes.rs:75`); `http_get` at `API_tests/run_api_tests.sh:1013` |
| `POST /api/v1/delegations` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `delegation_handler::create` (`src/routes.rs:76`); `http_post` at `API_tests/run_api_tests.sh:1017` |
| `POST /api/v1/delegations/{id}/revoke` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `delegation_handler::revoke` (`src/routes.rs:77`); `http_post` at `API_tests/run_api_tests.sh:1023` |
| `GET /api/v1/approvals` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `approval_handler::list` (`src/routes.rs:82`); `http_get` at `API_tests/run_api_tests.sh:1031` |
| `POST /api/v1/approvals` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `approval_handler::create_approval_request` (`src/routes.rs:83`); `http_post` at `API_tests/run_api_tests.sh:605` |
| `GET /api/v1/approvals/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `approval_handler::get` (`src/routes.rs:84`); `http_get` at `API_tests/run_api_tests.sh:1036` |
| `POST /api/v1/approvals/{id}/approve` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `approval_handler::approve` (`src/routes.rs:85`); `http_post` at `API_tests/run_api_tests.sh:465` |
| `POST /api/v1/approvals/{id}/reject` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `approval_handler::reject` (`src/routes.rs:86`); `http_post` at `API_tests/run_api_tests.sh:801` |
| `POST /api/v1/orders` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `order_handler::create_order` (`src/routes.rs:91`); `http_post` at `API_tests/run_api_tests.sh:185` |
| `GET /api/v1/orders` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `order_handler::list_orders` (`src/routes.rs:92`); `http_get` at `API_tests/run_api_tests.sh:195` |
| `GET /api/v1/orders/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `order_handler::get_order` (`src/routes.rs:93`); `http_get` at `API_tests/run_api_tests.sh:191` |
| `PUT /api/v1/orders/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `order_handler::update_order` (`src/routes.rs:94`); `http_put` at `API_tests/run_api_tests.sh:198` |
| `POST /api/v1/orders/{id}/transition` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `order_handler::transition_order` (`src/routes.rs:95`); `http_post` at `API_tests/run_api_tests.sh:201` |
| `POST /api/v1/orders/{id}/payments` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `order_handler::add_payment` (`src/routes.rs:99`); `http_post` at `API_tests/run_api_tests.sh:212` |
| `GET /api/v1/orders/{id}/payments` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `order_handler::list_payments` (`src/routes.rs:103`); `http_get` at `API_tests/run_api_tests.sh:223` |
| `POST /api/v1/orders/{id}/receipts` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `order_handler::attach_receipt` (`src/routes.rs:107`); `curl@228` at `API_tests/run_api_tests.sh:232` |
| `POST /api/v1/orders/{id}/returns` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `return_handler::initiate_return` (`src/routes.rs:111`); `http_post` at `API_tests/run_api_tests.sh:689` |
| `POST /api/v1/orders/{id}/exchanges` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `return_handler::initiate_exchange` (`src/routes.rs:115`); `http_post` at `API_tests/run_api_tests.sh:694` |
| `POST /api/v1/orders/{id}/reversals` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `return_handler::initiate_reversal` (`src/routes.rs:119`); `http_post` at `API_tests/run_api_tests.sh:450` |
| `POST /api/v1/orders/{id}/reversals/execute` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `return_handler::execute_reversal` (`src/routes.rs:123`); `http_post` at `API_tests/run_api_tests.sh:461` |
| `POST /api/v1/registers/close` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `register_handler::close_register` (`src/routes.rs:131`); `http_post` at `API_tests/run_api_tests.sh:1057` |
| `GET /api/v1/registers/closings` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `register_handler::list_closings` (`src/routes.rs:132`); `http_get` at `API_tests/run_api_tests.sh:1063` |
| `GET /api/v1/registers/closings/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `register_handler::get_closing` (`src/routes.rs:136`); `http_get` at `API_tests/run_api_tests.sh:1068` |
| `POST /api/v1/registers/closings/{id}/confirm` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `register_handler::confirm_closing` (`src/routes.rs:140`); `http_post` at `API_tests/run_api_tests.sh:1090` |
| `POST /api/v1/participants` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `participant_handler::create` (`src/routes.rs:148`); `http_post` at `API_tests/run_api_tests.sh:243` |
| `GET /api/v1/participants` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `participant_handler::list` (`src/routes.rs:149`); `http_get` at `API_tests/run_api_tests.sh:252` |
| `POST /api/v1/participants/bulk/tag` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `participant_handler::bulk_tag` (`src/routes.rs:150`); `http_post` at `API_tests/run_api_tests.sh:276` |
| `POST /api/v1/participants/bulk/deactivate` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `participant_handler::bulk_deactivate` (`src/routes.rs:151`); `http_post` at `API_tests/run_api_tests.sh:282` |
| `GET /api/v1/participants/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `participant_handler::get` (`src/routes.rs:155`); `http_get` at `API_tests/run_api_tests.sh:248` |
| `PUT /api/v1/participants/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `participant_handler::update` (`src/routes.rs:156`); `http_put` at `API_tests/run_api_tests.sh:260` |
| `DELETE /api/v1/participants/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `participant_handler::deactivate` (`src/routes.rs:157`); `http_delete` at `API_tests/run_api_tests.sh:1138` |
| `GET /api/v1/participants/{id}/tags` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `participant_handler::get_tags` (`src/routes.rs:158`); `http_get` at `API_tests/run_api_tests.sh:1099` |
| `PUT /api/v1/participants/{id}/tags` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `participant_handler::set_tags` (`src/routes.rs:159`); `http_put` at `API_tests/run_api_tests.sh:1104` |
| `POST /api/v1/participants/{id}/attachments` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `attachment_handler::upload` (`src/routes.rs:160`); `curl@1110` at `API_tests/run_api_tests.sh:1113` |
| `GET /api/v1/participants/{id}/attachments` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `attachment_handler::list` (`src/routes.rs:164`); `http_get` at `API_tests/run_api_tests.sh:1122` |
| `GET /api/v1/participants/{id}/attachments/{attachment_id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `attachment_handler::download` (`src/routes.rs:168`); `http_get` at `API_tests/run_api_tests.sh:1127` |
| `DELETE /api/v1/participants/{id}/attachments/{attachment_id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `attachment_handler::delete` (`src/routes.rs:172`); `http_delete` at `API_tests/run_api_tests.sh:1131` |
| `POST /api/v1/teams` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `team_handler::create` (`src/routes.rs:180`); `http_post` at `API_tests/run_api_tests.sh:264` |
| `GET /api/v1/teams` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `team_handler::list` (`src/routes.rs:181`); `http_get` at `API_tests/run_api_tests.sh:1146` |
| `GET /api/v1/teams/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `team_handler::get` (`src/routes.rs:182`); `http_get` at `API_tests/run_api_tests.sh:272` |
| `PUT /api/v1/teams/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `team_handler::update` (`src/routes.rs:183`); `http_put` at `API_tests/run_api_tests.sh:1151` |
| `DELETE /api/v1/teams/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `team_handler::deactivate` (`src/routes.rs:184`); `http_delete` at `API_tests/run_api_tests.sh:1168` |
| `GET /api/v1/teams/{id}/members` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `team_handler::list_members` (`src/routes.rs:185`); `http_get` at `API_tests/run_api_tests.sh:1156` |
| `POST /api/v1/teams/{id}/members` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `team_handler::add_member` (`src/routes.rs:186`); `http_post` at `API_tests/run_api_tests.sh:268` |
| `DELETE /api/v1/teams/{id}/members/{participant_id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `team_handler::remove_member` (`src/routes.rs:187`); `http_delete` at `API_tests/run_api_tests.sh:1161` |
| `POST /api/v1/datasets` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::create_dataset` (`src/routes.rs:195`); `http_post` at `API_tests/run_api_tests.sh:289` |
| `GET /api/v1/datasets` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::list_datasets` (`src/routes.rs:196`); `http_get` at `API_tests/run_api_tests.sh:1176` |
| `GET /api/v1/datasets/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::get_dataset` (`src/routes.rs:197`); `http_get` at `API_tests/run_api_tests.sh:1181` |
| `PUT /api/v1/datasets/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::update_dataset` (`src/routes.rs:198`); `http_put` at `API_tests/run_api_tests.sh:1186` |
| `DELETE /api/v1/datasets/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::deactivate_dataset` (`src/routes.rs:199`); `http_delete` at `API_tests/run_api_tests.sh:1209` |
| `POST /api/v1/datasets/{id}/versions` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::create_version` (`src/routes.rs:201`); `http_post` at `API_tests/run_api_tests.sh:293` |
| `GET /api/v1/datasets/{id}/versions` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::list_versions` (`src/routes.rs:205`); `http_get` at `API_tests/run_api_tests.sh:303` |
| `GET /api/v1/datasets/{id}/versions/{version_id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::get_version` (`src/routes.rs:209`); `http_get` at `API_tests/run_api_tests.sh:306` |
| `GET /api/v1/datasets/{id}/versions/{version_id}/lineage` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::get_lineage` (`src/routes.rs:214`); `http_get` at `API_tests/run_api_tests.sh:310` |
| `GET /api/v1/datasets/{id}/versions/{version_id}/fields` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::list_field_dictionary` (`src/routes.rs:219`); `http_get` at `API_tests/run_api_tests.sh:314` |
| `POST /api/v1/datasets/{id}/versions/{version_id}/fields` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::add_field_entry` (`src/routes.rs:223`); `http_post` at `API_tests/run_api_tests.sh:1191` |
| `PUT /api/v1/datasets/{id}/versions/{version_id}/fields/{field_id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::update_field_entry` (`src/routes.rs:227`); `http_put` at `API_tests/run_api_tests.sh:1197` |
| `DELETE /api/v1/datasets/{id}/versions/{version_id}/fields/{field_id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::delete_field_entry` (`src/routes.rs:231`); `http_delete` at `API_tests/run_api_tests.sh:1202` |
| `POST /api/v1/datasets/{id}/rollback` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::rollback` (`src/routes.rs:236`); `http_post` at `API_tests/run_api_tests.sh:318` |
| `POST /api/v1/datasets/{id}/rollback/execute` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `dataset_handler::execute_rollback` (`src/routes.rs:240`); `http_post` at `API_tests/run_api_tests.sh:1250` |
| `POST /api/v1/notification-templates` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::create_template` (`src/routes.rs:248`); `http_post` at `API_tests/run_api_tests.sh:1259` |
| `GET /api/v1/notification-templates` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::list_templates` (`src/routes.rs:249`); `http_get` at `API_tests/run_api_tests.sh:1265` |
| `GET /api/v1/notification-templates/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::get_template` (`src/routes.rs:250`); `http_get` at `API_tests/run_api_tests.sh:1270` |
| `PUT /api/v1/notification-templates/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::update_template` (`src/routes.rs:251`); `http_put` at `API_tests/run_api_tests.sh:1275` |
| `DELETE /api/v1/notification-templates/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::delete_template` (`src/routes.rs:252`); `http_delete` at `API_tests/run_api_tests.sh:1280` |
| `POST /api/v1/notifications/send` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::send_templated` (`src/routes.rs:261`); `http_post` at `API_tests/run_api_tests.sh:1294` |
| `POST /api/v1/notifications/send-direct` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::send_direct` (`src/routes.rs:262`); `http_post` at `API_tests/run_api_tests.sh:326` |
| `POST /api/v1/notifications/broadcast` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::broadcast` (`src/routes.rs:266`); `http_post` at `API_tests/run_api_tests.sh:345` |
| `GET /api/v1/notifications/inbox` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::inbox` (`src/routes.rs:271`); `http_get` at `API_tests/run_api_tests.sh:331` |
| `GET /api/v1/notifications/inbox/unread-count` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::unread_count` (`src/routes.rs:272`); `http_get` at `API_tests/run_api_tests.sh:335` |
| `POST /api/v1/notifications/inbox/read-all` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::mark_all_read` (`src/routes.rs:276`); `http_post` at `API_tests/run_api_tests.sh:342` |
| `GET /api/v1/notifications/inbox/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::get_notification` (`src/routes.rs:280`); `http_get` at `API_tests/run_api_tests.sh:1300` |
| `POST /api/v1/notifications/inbox/{id}/read` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::mark_read` (`src/routes.rs:284`); `http_post` at `API_tests/run_api_tests.sh:339` |
| `GET /api/v1/notifications/admin` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::admin_list` (`src/routes.rs:289`); `http_get` at `API_tests/run_api_tests.sh:1305` |
| `GET /api/v1/notifications/admin/{id}/delivery-logs` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::delivery_logs` (`src/routes.rs:290`); `http_get` at `API_tests/run_api_tests.sh:1310` |
| `POST /api/v1/notifications/admin/{id}/retry` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `notification_handler::retry` (`src/routes.rs:294`); `http_post` at `API_tests/run_api_tests.sh:1318` |
| `GET /api/v1/reports/kpi-types` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::list_kpi_types` (`src/routes.rs:302`); `http_get` at `API_tests/run_api_tests.sh:353` |
| `POST /api/v1/reports` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::create_definition` (`src/routes.rs:303`); `http_post` at `API_tests/run_api_tests.sh:357` |
| `GET /api/v1/reports` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::list_definitions` (`src/routes.rs:304`); `http_get` at `API_tests/run_api_tests.sh:369` |
| `GET /api/v1/reports/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::get_definition` (`src/routes.rs:305`); `http_get` at `API_tests/run_api_tests.sh:1333` |
| `PUT /api/v1/reports/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::update_definition` (`src/routes.rs:306`); `http_put` at `API_tests/run_api_tests.sh:1338` |
| `DELETE /api/v1/reports/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::delete_definition` (`src/routes.rs:307`); `http_delete` at `API_tests/run_api_tests.sh:1346` |
| `POST /api/v1/reports/{id}/run` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::run_report` (`src/routes.rs:308`); `http_post` at `API_tests/run_api_tests.sh:364` |
| `POST /api/v1/scheduled-reports` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::create_schedule` (`src/routes.rs:313`); `http_post` at `API_tests/run_api_tests.sh:372` |
| `GET /api/v1/scheduled-reports` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::list_schedules` (`src/routes.rs:314`); `http_get` at `API_tests/run_api_tests.sh:1350` |
| `GET /api/v1/scheduled-reports/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::get_schedule` (`src/routes.rs:315`); `http_get` at `API_tests/run_api_tests.sh:1360` |
| `PUT /api/v1/scheduled-reports/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::update_schedule` (`src/routes.rs:316`); `http_put` at `API_tests/run_api_tests.sh:1365` |
| `DELETE /api/v1/scheduled-reports/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `report_handler::delete_schedule` (`src/routes.rs:317`); `http_delete` at `API_tests/run_api_tests.sh:1370` |
| `POST /api/v1/exports` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `export_handler::request_export` (`src/routes.rs:322`); `http_post` at `API_tests/run_api_tests.sh:376` |
| `GET /api/v1/exports` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `export_handler::list_jobs` (`src/routes.rs:323`); `http_get` at `API_tests/run_api_tests.sh:384` |
| `GET /api/v1/exports/admin` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `export_handler::admin_list_jobs` (`src/routes.rs:324`); `http_get` at `API_tests/run_api_tests.sh:1378` |
| `GET /api/v1/exports/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `export_handler::get_job` (`src/routes.rs:325`); `http_get` at `API_tests/run_api_tests.sh:380` |
| `PUT /api/v1/exports/{id}/progress` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `export_handler::update_progress` (`src/routes.rs:326`); `http_put` at `API_tests/run_api_tests.sh:1392` |
| `POST /api/v1/exports/{id}/complete` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `export_handler::complete_job` (`src/routes.rs:330`); `http_post` at `API_tests/run_api_tests.sh:549` |
| `POST /api/v1/exports/{id}/fail` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `export_handler::fail_job` (`src/routes.rs:334`); `http_post` at `API_tests/run_api_tests.sh:1408` |
| `POST /api/v1/exports/{id}/cancel` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `export_handler::cancel_job` (`src/routes.rs:335`); `http_post` at `API_tests/run_api_tests.sh:1418` |
| `GET /api/v1/exports/{id}/download` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `export_handler::download_export` (`src/routes.rs:339`); `http_get` at `API_tests/run_api_tests.sh:554` |
| `GET /api/v1/audit` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `audit_handler::list` (`src/routes.rs:347`); `http_get` at `API_tests/run_api_tests.sh:394` |
| `GET /api/v1/audit/{id}` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `audit_handler::get` (`src/routes.rs:348`); `http_get` at `API_tests/run_api_tests.sh:1431` |
| `GET /api/v1/health` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `health::health_check` (`src/routes.rs:351`); `curl@97` at `API_tests/run_api_tests.sh:97` |
| `GET /api/v1/metrics` | yes | true no-mock HTTP | `API_tests/run_api_tests.sh` | `health::metrics_endpoint` (`src/routes.rs:352`); `http_get` at `API_tests/run_api_tests.sh:406` |

## API Test Classification
- **True No-Mock HTTP:** 1 suite (`API_tests/run_api_tests.sh`) covering 129 / 129 endpoints via real HTTP (`curl`) against running service.
- **HTTP with Mocking:** none found.
- **Non-HTTP (unit/integration without HTTP):** Rust unit tests embedded with `#[cfg(test)]` in 12 source files.

## Mock Detection
- No `jest.mock`, `vi.mock`, `sinon.stub`, or shell-level mocking patterns detected in test artifacts (`API_tests/run_api_tests.sh`, `src/**/*.rs`).
- No Actix in-process HTTP test harness (`actix_web::test`, `TestRequest`, `init_service`) detected; HTTP validation is external black-box via `curl`.

## Coverage Summary
- Total endpoints: **129**
- Endpoints with HTTP tests: **129**
- Endpoints with TRUE no-mock HTTP tests: **129**
- HTTP coverage: **100.00%**
- True API coverage: **100.00%**

## Unit Test Summary
### Backend Unit Tests
- Test files detected: `src/auth/password.rs`, `src/auth/jwt.rs`, `src/auth/lockout.rs`, `src/crypto/aes.rs`, `src/crypto/masking.rs`, `src/pos/state_machine.rs`, `src/rbac/guard.rs`, `src/storage/mod.rs`, `src/audit/service.rs`, `src/observability/metrics.rs`, `src/models/order_line_item.rs`, `src/errors.rs`.
- Modules covered: auth helpers, crypto utilities, state machine logic, RBAC path matching, storage safety/utilities, audit hashing, metrics helper, error responses, one model validator.
- Important backend modules NOT unit-tested: `src/handlers/*.rs`, `src/db.rs`, `src/main.rs`, `src/routes.rs`, `src/pos/idempotency.rs`, `src/models/order.rs`, `src/models/dataset.rs`.

### Frontend Unit Tests (STRICT REQUIREMENT)
- Frontend test files: **NONE**
- Frameworks/tools detected: **NONE**
- Frontend components/modules covered: **NONE**
- Important frontend components/modules not tested: **N/A (backend project; no frontend code detected)**
- **Frontend unit tests: MISSING**
- CRITICAL GAP trigger check: not triggered because project type is backend.

### Cross-Layer Observation
- No frontend layer detected in repository (`package.json` absent; no `*.tsx/*.jsx/*.js` app files).

## API Observability Check
- Endpoint method+path and request input are explicit in `API_tests/run_api_tests.sh` via helper calls and curl.
- Response checks are partly weak in status-only assertions.

## Tests Check
- Success/failure/edge/auth paths are broadly represented in API tests.
- Integration boundaries are real HTTP but many assertions are shallow (status/substrings).
- `run_tests.sh` is Docker-based, but includes runtime package/tool installation (`apt-get`, `cargo install cargo-llvm-cov`) and therefore not fully hermetic.

## End-to-End Expectations
- Project type is backend; FE↔BE end-to-end expectation not mandatory.

## Test Coverage Score (0-100)
**88/100**

## Score Rationale
- Very high endpoint coverage with real HTTP path.
- Breadth is strong; depth is moderate due assertion quality and limited handler/service unit tests.

## Key Gaps
- Limited unit-level coverage of handlers and orchestration.
- API response contract assertions are not strict enough.
- Test reproducibility depends on runtime installs.

## Confidence & Assumptions
- Confidence: **High** for endpoint mapping.
- Assumption: no additional runtime-registered routes outside `routes::configure`.
- Static inspection only; no tests executed.

## Test Coverage Verdict
**PASS WITH GAPS**

---

# README Audit

## Hard Gate Check
- README location: **PASS** (`repo/README.md` exists).
- Markdown formatting/readability: **PASS**.
- Startup instructions include `docker-compose up`: **PASS** (`repo/README.md:15`).
- Access method (URL + port): **PASS** (e.g., `localhost:8081`).
- Verification method: **PASS** (curl examples).
- Environment rules: **PASS** (no local package-manager install steps in README).
- Demo credentials for auth system: **PASS** (`repo/README.md:29-35`).

## High Priority Issues
- None.

## Medium Priority Issues
- `jq` is used in commands (`repo/README.md:43-45`) but not listed as prerequisite.
- Hardcoded test counts in README may drift (`repo/README.md:97-99`).

## Low Priority Issues
- Some duplication in long compliance narrative sections.

## Hard Gate Failures
- None.

## README Verdict
**PASS**

## Final Combined Verdict
- **Test Coverage Audit:** PASS WITH GAPS
- **README Audit:** PASS
