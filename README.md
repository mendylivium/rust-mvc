# RustMVC

A Laravel-inspired Rust starter with a ready-to-use Vue admin workspace. Inspired by the directory structure and Forge workflow of [mendylivium/go-mvc](https://github.com/mendylivium/go-mvc).

**Axum · SQLx / SQLite, MySQL, PostgreSQL · Vue 3 · Vue Router · Vite · Tailwind CSS 4 · daisyUI 5**

The frontend is a Vue SPA talking to a same-origin JSON API. Unlike the Go reference's Inertia integration, navigation lives in Vue Router. Rust handles validation, authorization, persistence, and authentication. This is a focused starter, not a feature-for-feature port of GoMVC or Laravel.

## Get started

Requirements: current stable Rust (verified with 1.99), a C compiler/linker, Node.js 22.12+ (24 LTS recommended), and npm. Install Rust with [rustup](https://rustup.rs/) if `cargo --version` is unavailable. SQLite is bundled; no database service is needed.

```sh
cp .env.example .env
npm ci
cargo fetch
```

Set `SEED_ADMIN_PASSWORD` in `.env` to your own password of 12–128 bytes. The seed email defaults to `admin@example.com`; change `SEED_ADMIN_EMAIL` if desired. There is no built-in default password.

```sh
./forge migrate --seed
make dev
```

Open **http://127.0.0.1:5173** and sign in using your seed email/password. Vite proxies `/api` to Rust on port 3000, preserving same-origin cookies. Use `127.0.0.1` consistently. The seed command is idempotent and never changes an existing account's password.

Without Make, use `node scripts/dev.mjs`, or start these in separate terminals:

```sh
./forge serve
npm run dev
```

On Windows, use `forge.cmd` instead of `./forge`. Rust compilation happens automatically on `forge` commands; restart the backend after Rust changes. Vue/CSS changes have Vite HMR. If changing the backend port, update the proxy in `vite.config.js` as well.

## Tutorial and UI examples

Follow the [complete tutorial](docs/TUTORIAL.md) for setup, request flow, a database-backed Projects feature, UI customization, testing, deployment, and troubleshooting.

After signing in, open **Getting started** in the sidebar. The **Elements & buttons**, **Cards**, **Tables**, and **Forms** pages provide interactive examples you can copy from `resources/js/pages/examples/`. Examples use local sample data; Users and Settings persist through the Rust API. All examples are available to signed-in administrators and members.

## Included

- Responsive admin layout, mobile navigation, login, overview, searchable and paginated user management, add/edit/delete dialogs, and persistent profile settings.
- Dashboard statistics and six-month registration chart sourced from the database. Monthly reporting uses UTC. New workspaces start with one seeded admin; no fabricated activity or placeholder statistics.
- Administrator/member roles enforced on the server. Members can sign in and edit their own name, while administrators can manage users. Self-deletion, self-demotion, and self-deactivation are blocked.
- Argon2id password hashing, opaque database sessions with a 12-hour absolute lifetime, HttpOnly/SameSite cookies, synchronizer CSRF tokens, session rotation on login, session revocation on password changes/deactivation, and per-account login throttling (5 attempts per 15 minutes).
- Typed configuration, form-request validation, controller/model/policy separation, migrations, seeders, JSON errors, request logging, graceful shutdown, generators, CI, and a Dockerfile.
- All UI corner radii are at most **0.25rem**. Theme tokens and custom styles live in `resources/css/app.css`.

## Project layout

```text
app/
  console/                 # forge commands and code generators
  http/
    controllers/           # auth, dashboard, and user actions
    middleware/            # sessions, authentication extractor, CSRF
    requests/              # validated form input
    error.rs               # JSON error responses
  models/                  # SQLx models
  policies/                # authorization rules
bootstrap/                 # crate root, shared state, HTTP application
config/                    # .env → typed configuration
database/
  migrations/              # versioned SQL migrations embedded in the binary
  seeders/                 # idempotent administrator creation
routes/
  api.rs                   # JSON API endpoints
  web.rs                   # production static files and SPA fallback
resources/
  css/app.css              # Tailwind + daisyUI theme and layout
  js/
    components/            # shared UI and accessible native dialogs
    layouts/               # admin shell
    pages/                 # app pages, UI examples, getting started, 404
    lib/                   # API client, auth state, formatters
    router.js              # navigation and route guards
public/                    # assets copied by Vite into dist/
storage/                   # ignored runtime database files
tests/                     # Rust HTTP integration + Playwright browser tests
main.rs                    # executable entry point
forge / forge.cmd          # cargo-backed CLI wrappers
```

Rust modules use `mod.rs` and explicit `#[path]` declarations in `bootstrap/lib.rs` to keep the Laravel-style root layout. There is no duplicate `src/` tree.

## Forge commands

```sh
./forge serve
./forge migrate
./forge migrate --seed
./forge db:seed
./forge make:controller product_controller
./forge make:model product
./forge --help
```

Generators accept snake_case identifiers and refuse to overwrite files. They print the module registration step. Register the generated module in its parent `mod.rs`, then wire the controller in `routes/api.rs` or add a migration for the model. Generated models are SQLx row structs, not active-record objects.

Create new migration files with a unique increasing numeric prefix, for example `database/migrations/20261016000001_create_products.sql`. Run `./forge migrate`. Applied migrations should never be edited. This starter intentionally includes forward-only migrations; rollbacks, queues, mail, JWT/mobile API auth, password recovery, and Inertia are extension work.

## API

All endpoints below are under `/api`. Read `/auth/session` first to obtain the cookie and `csrf_token`. Send `X-CSRF-Token` on every POST/PUT/PATCH/DELETE request. The browser client does this automatically; do not store session credentials in localStorage.

| Method | Path                          | Access                                      |
| ------ | ----------------------------- | ------------------------------------------- |
| GET    | `/health`                     | Public liveness check                       |
| GET    | `/auth/session`               | Public; returns user or null and CSRF token |
| POST   | `/auth/login`                 | Guest credentials + CSRF                    |
| POST   | `/auth/logout`                | Session + CSRF                              |
| PATCH  | `/profile`                    | Signed-in user; own name only               |
| GET    | `/dashboard`                  | Administrator                               |
| GET    | `/users?search=&role=&page=1` | Administrator; 10 per page                  |
| POST   | `/users`                      | Administrator                               |
| PUT    | `/users/{id}`                 | Administrator                               |
| DELETE | `/users/{id}`                 | Administrator                               |

User writes accept `name`, `email`, `password`, `role` (`admin`/`member`), and `status` (`active`/`inactive`). On update, an empty password keeps the existing hash. Validation failures return HTTP 422 with `{ "message": "...", "errors": { "field": ["..."] } }`. Unknown API routes return JSON 404 responses. Authentication failure is 401 and insufficient permissions/CSRF failure is 403.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
npm run build
npx playwright install chromium
npm run test:e2e
```

Browser tests start an isolated Rust server on port 3100 with a temporary SQLite database. They do not modify your development database. Build the frontend before running them. Screenshots and failed-test traces are saved under `test-results/` (ignored). CI installs browser system dependencies as well.

## Database selection

Choose `DB_CONNECTION=sqlite`, `mysql`, or `pgsql` in `.env`, then configure `DB_HOST`, `DB_PORT`, `DB_DATABASE`, `DB_USERNAME`, and `DB_PASSWORD`. Run `./forge migrate` and restart the backend. A nonempty `DATABASE_URL` overrides `DB_*`; remove an old URL when switching. Switching connections does not move existing data.

See [database configuration and Compose services](docs/DATABASES.md) for complete examples, TLS, migration conventions, and backend tests.

## Standalone server and desktop

Build a browser-based server binary or a native desktop window with embedded frontend assets:

```sh
make build MODE=server TARGET=linux
make build MODE=desktop TARGET=linux
# On Windows, without Make:
npm run build:desktop -- --target windows
```

Build on the matching OS/architecture. Outputs go to `releases/`. Desktop uses Tauri IPC without an HTTP listener and includes first-run administrator setup. See [standalone builds and deployment](docs/STANDALONE.md) for prerequisites, Windows/Linux artifacts, app-data locations, and server setup.

## Docker Compose

See the [Docker Compose deployment tutorial](docs/DOCKER.md) for configuration, first-admin setup, HTTPS, updates, backups, and restore.

```sh
# Create .env from .env.example first if you do not already have one.
docker compose up -d --build --wait --wait-timeout 180
```

Open **http://127.0.0.1:8080**. Follow the tutorial to seed your first administrator. The included `docker-compose.yml` applies migrations before starting the app, checks HTTP health, and persists SQLite in a named volume. The container database is separate from local development storage.

## Production

```sh
make build
./releases/server/linux-x64/rustmvc-server migrate
./releases/server/linux-x64/rustmvc-server serve
```

Run from the deployment directory so `.env` and `storage/` resolve correctly. The example output path above assumes Linux x64. A standalone server build embeds the frontend and migrations; deploy `releases/server/<platform>/rustmvc-server` (or `.exe`). A plain `cargo build --release` without `embedded-assets` still needs a sibling `dist/` directory. Supply configuration as environment variables or a private `.env`, use HTTPS with `COOKIE_SECURE=true`, and persist/back up `storage/`. Set `APP_HOST=0.0.0.0` when serving through a container or external reverse proxy. The built Rust server serves both the SPA and API on port 3000, including direct navigation to `/users` and `/settings`.

The Dockerfile builds both layers and runs as an unprivileged user. Mount writable persistent storage at `/app/storage`, run `forge migrate` (and optionally `forge db:seed` with seed credentials) before starting the serving container. Docker deployment is optional; the Docker image itself is not part of the local verification run.

HTTP authentication has a shared budget of 30 login attempts/minute per process, plus 5 attempts per email per 15 minutes. At most 4 password operations run concurrently; additional work is rejected with HTTP 429 instead of queued. Creating new anonymous sessions is limited to 60/minute per process, with a database cap of 10,000 total sessions. Existing session reads and logout do not require new-session capacity. Login attempt records are capped at 10,000 and pruned after 15 minutes; unknown and registered addresses follow the same throttle. Expired sessions are pruned on session creation.

The process budgets live in `app/http/middleware/security.rs`. Tune them for your application's traffic, and add edge/client-based limits for fairness in public deployments. The application does not trust forwarded IP headers. Keep SQLite for single-instance workloads; MySQL/PostgreSQL provide server database options but do not share process budgets; process budgets reset on restart and are not shared across replicas, while database caps and account counters persist.

Password verification is followed by a transactional credential check and session rotation. A password reset either invalidates a login's stale credential snapshot or revokes the issued session. Profile edits without a new password preserve the current database hash atomically.

Production SPA routes `/`, `/login`, `/users`, `/settings`, `/guide`, and `/examples/{elements,cards,tables,forms}` return HTTP 200; missing assets and unknown routes keep HTTP 404. Register new SPA paths in both `resources/js/router.js` and `routes/web.rs`. JSON/body/path/query rejection errors use the same JSON envelope as controller errors. Expired-session writes return 401. A CSRF mismatch refreshes client session state and asks for another submission; writes are never automatically replayed.

Apply `./forge migrate` when updating this starter to add the login-attempt expiry index. Read [AUDIT.md](AUDIT.md) for the original findings and remediation verification. RustSec reports RUSTSEC-2023-0071 for RSA, now an active dependency through the MySQL driver; no advisory suppression is configured.

## Reference documentation

- [Axum](https://docs.rs/axum/latest/axum/)
- [SQLx](https://docs.rs/sqlx/latest/sqlx/)
- [Vue](https://vuejs.org/guide/quick-start.html)
- [daisyUI + Vite](https://daisyui.com/docs/install/vite/)
- [daisyUI themes](https://daisyui.com/docs/themes/)
