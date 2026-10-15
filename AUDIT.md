# Full audit — 14 October 2026

Audited commit: `af69298` (`Initial Commit`). Application source was not changed during this audit. Runtime probes used isolated SQLite databases under `/tmp/rust-mvc-audit/`; the development database was not used.

## Remediation — 14 October 2026

The eight source findings below are historical findings against `af69298`; all eight have now been addressed in the working tree.

| Finding | Resolution                                                                                                                                                                                                                                                                | Regression coverage                                                                                                               |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| A01     | Session issuance conditionally verifies the current password hash and active status inside the same write transaction as rotation. Blank-password updates use SQL `COALESCE` to preserve the current hash.                                                                | Controlled reset/issuance ordering tests cover reset-before-login and login-before-reset.                                         |
| A02     | Shared 30/min login and 60/min new-session budgets, 4 fail-fast password-work slots, 10,000-row session/attempt caps, and indexed expiry cleanup. Account throttling treats unknown and existing emails consistently. Logout does not require anonymous-session capacity. | Admission saturation, rotating email input, storage caps/reclamation, unknown-email throttling, and logout under exhausted quota. |
| A03     | Explicit production routes serve the SPA with 200; missing assets, unknown pages, and unknown APIs remain 404.                                                                                                                                                            | Browser tests assert response status, not only rendered content.                                                                  |
| A04     | JSON, path, and query extractors map rejections into the application JSON envelope.                                                                                                                                                                                       | Missing fields, malformed JSON, invalid path/query values, and oversized body tests.                                              |
| A05     | Muted text colors have sufficient contrast across both light and dark surfaces.                                                                                                                                                                                           | axe-core WCAG 2/2.1 A/AA scans pass for login, dashboard, users, settings, and the user dialog.                                   |
| A06     | Closed mobile navigation is inert/hidden; the open menu manages focus, wraps Tab navigation, closes on Escape, and restores the trigger's focus.                                                                                                                          | Mobile keyboard and desktop breakpoint tests.                                                                                     |
| A07     | Seed email uses the same normalization and validation as user requests, before hashing/insertion.                                                                                                                                                                         | Empty/malformed emails rejected; normalization and existing-account password preservation checked.                                |
| A08     | Both input and generated identifiers are parsed as Rust identifiers; the Rust 2024 `gen` keyword is explicitly rejected.                                                                                                                                                  | Reserved/transformed names rejected without writing files; valid generation and overwrite protection checked.                     |

Additional fixes: expired-session mutations now return 401; CSRF mismatches refresh client state without automatically replaying writes. Cross-tab tests verify that the first submission is rejected without changing data and only an explicit second submission saves it.

**Post-fix verification:** 13 Rust tests (11 HTTP and 2 CLI), 5 Chromium browser tests, Rust formatting, Clippy with warnings denied, frontend build, and optimized Rust build. npm audit reports zero vulnerabilities. The 0.25rem corner-radius requirement is retained and tested. Docker image execution remains unverified.

**Dependency limitation:** the RustSec scan still flags `RUSTSEC-2023-0071` in the locked optional RSA dependency. SQLx now has explicit default-disabled feature selection, but its optional lockfile family still includes RSA. `cargo tree --locked -i rsa` confirms no active path in the SQLite/Linux build. No advisory has been suppressed, and no upstream patched version was reported. Re-evaluate this before enabling other database drivers.

Deployment/update step: run `./forge migrate` for the new login-attempt expiry index. Rate budgets are per process and shared across callers; retain suitable edge/client-based limits for fairness, and tune the constants in `app/http/middleware/security.rs` for expected traffic. Persistent database caps/account counters survive restart; in-memory admission budgets do not.

## Original assessment

**Do not treat the current passing tests as production readiness.** A password-reset/login race was reproduced using the application's default password hashing and public HTTP endpoints. There are also production routing, API contract, accessibility, and tooling defects.

Eight source findings: **2 high, 4 medium, 2 low**. The second high finding is an availability exposure for publicly reachable deployments without an upstream limiter; the audit did not attempt to exhaust the machine.

## Findings

### A01 — High: a login using the old password can survive a completed password reset

Location: [auth_controller.rs](app/http/controllers/auth_controller.rs#L66), particularly session creation at line 102; [user_controller.rs](app/http/controllers/user_controller.rs#L125).

Login reads the password hash and user status before awaiting password verification. Meanwhile, an administrator can update the password and delete all existing sessions. Login then creates a new session using its stale snapshot, without rechecking the credentials inside the same transaction as session creation. The new session is created after the reset's revocation transaction and remains valid for up to 12 hours.

**Reproduction:** create a member through `POST /api/users` with the normal default Argon2 hash. Start an administrator's password-reset request. About 100 ms later, start the member's login with the old password. The observed reset returned HTTP 200 at 0.958 s; the old-password login returned HTTP 200 at 1.116 s; a subsequent authenticated `PATCH /api/profile` also returned HTTP 200. No custom hash or database mutation was required for this reproduction. An earlier higher-cost-hash fixture also reproduced the race, but the default-configuration reproduction is the basis for this finding.

**Fix:** make session issuance conditional on the exact verified hash or credential version still matching the active user, atomically with the session insert. Serialize that decision with password-reset/session-revocation writes. Add a controlled concurrency regression test that fails when an old credential can create a usable session after reset.

### A02 — High: changing the email bypasses the only limit on expensive anonymous login work

Location: [auth_controller.rs](app/http/controllers/auth_controller.rs#L57), [session.rs](app/http/middleware/session.rs#L51), [application.rs](bootstrap/application.rs#L12).

The throttle is keyed solely by user-supplied email. Each new address gets a fresh allowance, a persistent `login_attempts` row, and an expensive Argon2 operation even when no user exists. `spawn_blocking` is used without an application-level semaphore or bounded admission queue. Anonymous session creation also performs unthrottled database writes. A client can obtain its own anonymous CSRF token, so CSRF does not mitigate this resource-consumption path.

**Evidence:** six sequential attempts from one client to six distinct nonexistent addresses all returned 422 and left six attempt rows; none hit a global or client-based throttle. Source inspection confirms there is no complementary limiter or hash-work budget. No flood test or actual out-of-memory condition was attempted.

**Fix:** bound concurrent password work and queue length, add appropriate client/global admission limits before hashing, and cap/expire anonymous-session and attempt records. Correctly configure trusted proxies before using forwarded IP addresses. Retain per-account protection as an additional control. The README already mentions adding edge limits and cleaning attempt rows, but neither is supplied or enforced by the starter.

### A03 — Medium: valid SPA routes return HTTP 404 in production

Location: [web.rs](routes/web.rs#L7).

`ServeDir::not_found_service(ServeFile::new("dist/index.html"))` serves the HTML shell with a 404 status. Direct requests and reloads at `/login` and `/users` both returned **404** even though Vue mounted successfully. This can break monitoring, reverse-proxy error interception, and consumers relying on correct document status. Vite development navigation does not expose the same failure.

**Fix:** serve the HTML shell with HTTP 200 for recognized SPA navigation paths while preserving genuine 404 responses for missing assets and unknown API routes. Add assertions on the document response status to browser tests; the current tests only assert that headings become visible.

### A04 — Medium: extractor failures bypass the documented JSON error contract

Location: [user_controller.rs](app/http/controllers/user_controller.rs#L30), [user_controller.rs](app/http/controllers/user_controller.rs#L94), [api.js](resources/js/lib/api.js#L26).

The custom `AppError` response is bypassed when Axum's `Json`, `Path`, or `Query` extractor rejects a request. These failures return plain text, which the client discards while attempting `response.json()`.

**Observed authenticated requests:**

| Request                            | Status | Content type / result               |
| ---------------------------------- | ------ | ----------------------------------- |
| `GET /api/users?page=abc`          | 400    | `text/plain`; invalid query integer |
| `POST /api/users` with only `name` | 422    | `text/plain`; missing `email`       |
| `DELETE /api/users/not-an-id`      | 400    | `text/plain`; invalid path integer  |

This contradicts the documented validation envelope and loses useful field information in the UI/API client.

**Fix:** introduce rejection-mapping extractors or explicitly handle extractor rejections and return the same JSON envelope, with field-level errors where meaningful. Test malformed JSON, missing required fields, invalid path/query parameters, and oversized bodies.

### A05 — Medium: widespread text contrast failures

Location: [app.css](resources/css/app.css#L645), and the muted text colors used throughout the stylesheet.

An axe-core 4.14.0 WCAG 2/2.1 A/AA scan of the rendered desktop dashboard reported the `color-contrast` rule with 37 failing nodes in the audit fixture. Counts vary with user rows; the underlying stylesheet issue is consistent.

Examples include table headings at **3.05:1** (`#8b9386` on `#fafbf8`), supporting metric copy at **3.21:1**, and footer text at **2.51:1**. These are small normal-weight text elements requiring 4.5:1. Important table labels, email text, and explanatory copy are affected, not only decorative numbering.

**Fix:** darken the shared muted-text palette and validate each text/background combination. Keep the requested 0.25rem radius unchanged. Add accessibility scans for login, dashboard, users, settings, and open dialogs; the current radius/layout tests do not measure accessibility.

### A06 — Medium: the closed mobile navigation remains keyboard-focusable

Location: [AdminLayout.vue](resources/js/layouts/AdminLayout.vue#L33), [app.css](resources/css/app.css#L1100).

Below 900px the sidebar is translated out of view, but its links and buttons remain in the tab order and accessibility tree. At a 390px viewport, after waiting for the close transition to complete, the first Tab focused the hidden brand link at **x = -220px, right = -18px**. Opening the sidebar and pressing Escape also left it open. Focus is not managed as a mobile navigation overlay.

**Fix:** make the closed mobile sidebar inert/hidden to keyboard and assistive technology, move focus into it on opening, support Escape, and restore focus to the menu button on closing. Preserve normal desktop navigation. Add keyboard tests, including Tab behavior when closed and focus containment when open.

### A07 — Low: the seeder can create an administrator that cannot use the login form

Location: [seeders/mod.rs](database/seeders/mod.rs#L4).

The seed command validates only password length. With `SEED_ADMIN_EMAIL=not-an-email`, `forge migrate --seed` exited successfully and printed `Created administrator: not-an-email`. The browser's required `type="email"` field prevents this seeded identifier from being submitted. The backend technically accepts the identifier, but the documented first-login workflow fails.

**Fix:** reuse the email validation/normalization rules before inserting a seeded administrator and return a clear configuration error. Test invalid and empty seed emails as well as duplicate-account behavior.

### A08 — Low: accepted model names can generate invalid Rust

Location: [console/mod.rs](app/console/mod.rs#L42), [console/mod.rs](app/console/mod.rs#L61).

The generator checks the input identifier but not the PascalCase type it produces. `forge make:model self_` succeeds and generates `pub struct Self`, which Rust rejects with `expected identifier, found keyword Self`. The hand-maintained keyword list also omits reserved words such as `do`.

**Fix:** validate both the module identifier and generated type identifier against Rust's full keyword/reserved-identifier rules before writing a file. Test compilation/parsing of representative generated stubs, including transformed and reserved names.

## Additional observations

- **Session recovery UX:** logging out and back in from a second tab rotates the shared cookie but leaves the first tab's CSRF token stale in memory. The first tab's create-user request returns `Invalid CSRF token. Refresh and try again.` Expired sessions likewise return 403 from CSRF middleware, while the frontend's automatic session-expiry handling watches only 401. Manual refresh recovers; no authentication bypass was observed. Consider explicit session refresh/redirect handling without blindly replaying writes.
- **Concurrent user updates:** password hashes are read before the update transaction, including when keeping an unchanged password. Review stale-write behavior before adding concurrent administrative workflows; this audit did not reproduce a separate password-restoration race.
- **Positive controls observed:** parameter-bound SQL; password hashes omitted from serialized users; server-side admin policy checks; CSRF rejection; login session rotation; sequential logout and deactivation revocation; self-demotion/deletion prevention; responsive layout and corner-radius checks. These controls do not eliminate A01.
- No tracked `.env` or common private-key/AWS-access-key/GitHub-token patterns were found by the limited pattern scan. This is not a guarantee that every possible secret format was detected.

## Verification record

| Check                                                | Result                                                               |
| ---------------------------------------------------- | -------------------------------------------------------------------- |
| `cargo fmt --check`                                  | Passed                                                               |
| `cargo clippy --locked --all-targets -- -D warnings` | Passed                                                               |
| `cargo test --locked`                                | 5 integration tests passed                                           |
| `npm run build`                                      | Passed                                                               |
| `npm run test:e2e`                                   | 2 Chromium browser tests passed                                      |
| `npm audit --json`                                   | 0 reported vulnerabilities                                           |
| Prettier check                                       | Passed                                                               |
| Additional HTTP / browser probes                     | Findings above reproduced against an isolated database               |
| `cargo build --release --locked -j 2`                | Passed                                                               |
| RustSec dependency audit                             | 1 lockfile advisory: RSA; no active Linux build path (details below) |
| Docker image / hosted CI                             | Source reviewed; not executed                                        |

## RustSec result and reachability

`cargo-audit` 0.22.2 scanned 258 locked packages against the RustSec database at commit `6aed84668c23d7f535077a9fe918bf1fa0f6a5bc` (updated 13 October 2026). It exited with code 1 for **one advisory**, [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html), affecting locked `rsa 0.9.10` (Marvin timing attack). No patched version is listed by the advisory database.

This is a dependency-lock finding, **not a demonstrated vulnerability in the running SQLite application**. RSA appears in the lockfile through SQLx's optional MySQL dependency family; `cargo tree --locked -i rsa` reports no active path for the audited build, and the production build does not compile RSA. Track this advisory, re-evaluate reachability if enabling additional SQLx drivers, and document any narrowly scoped audit exception instead of claiming the Rust dependency scan is clean. It is listed separately from the eight application-source findings.

## Coverage limits and recommended order

Review covered the Rust app/configuration/migrations/CLI, Vue pages and shared client, stylesheet, development/test scripts, dependency locks, Dockerfile, and CI configuration. Browser execution used Chromium on Linux. Docker was unavailable; no container image, Windows/macOS behavior, deployed TLS/reverse proxy, or hosted CI run was validated. No destructive capacity test was performed.

Prioritize A01 and A02 before public deployment, then fix A03/A04 and add their regression tests. Address A05/A06 before treating the admin template as accessible. A07/A08 are isolated setup/tooling repairs. The existing tests pass because they exercise successful sequential flows and do not cover these concurrency, protocol, accessibility, or generator edge cases.


## Database-driver update

MySQL/PostgreSQL support now uses SQLx AnyPool. RUSTSEC-2023-0071 (RSA) is an active dependency through MySQL, superseding the earlier unused-lockfile assessment. No suppression is applied. Authentication storage/session transactions now serialize on a database auth_lock row to preserve bounded storage and password-reset ordering on all drivers. See docs/DATABASES.md for backend verification and deployment constraints.
