# RustMVC

A Laravel-inspired Rust starter with a ready-to-use Vue admin workspace. Inspired by the directory structure and Forge workflow of [mendylivium/go-mvc](https://github.com/mendylivium/go-mvc).

**Axum · SQLx / SQLite · Vue 3 · Vue Router · Vite · Tailwind CSS 4 · daisyUI 5**

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
    pages/                 # login, dashboard, users, settings, 404
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

Create new migration files with a unique increasing numeric prefix, for example `database/migrations/20261015000001_create_products.sql`. Run `./forge migrate`. Applied migrations should never be edited. This starter intentionally includes forward-only migrations; rollbacks, additional database drivers, queues, mail, JWT/mobile API auth, password recovery, and Inertia are extension work.

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

## Production

```sh
make build
./target/release/forge migrate
./target/release/forge serve
```

Run from the project/deployment root so `dist/` and `storage/` resolve correctly. Deploy the `forge` binary plus `dist/`; migrations are embedded. Supply configuration as environment variables or a private `.env`, use HTTPS with `COOKIE_SECURE=true`, and persist/back up `storage/`. Set `APP_HOST=0.0.0.0` when serving through a container or external reverse proxy. The built Rust server serves both the SPA and API on port 3000, including direct navigation to `/users` and `/settings`.

The Dockerfile builds both layers and runs as an unprivileged user. Mount writable persistent storage at `/app/storage`, run `forge migrate` (and optionally `forge db:seed` with seed credentials) before starting the serving container. Docker deployment is optional; the Docker image itself is not part of the local verification run.

The bundled login throttle is per account. Add edge/IP rate limits appropriate to your deployment, and schedule cleanup of old `login_attempts` rows for long-running public services. Expired sessions are pruned when creating sessions. Keep SQLite for single-instance workloads or adapt the database layer before scaling across hosts.

## Reference documentation

- [Axum](https://docs.rs/axum/latest/axum/)
- [SQLx](https://docs.rs/sqlx/latest/sqlx/)
- [Vue](https://vuejs.org/guide/quick-start.html)
- [daisyUI + Vite](https://daisyui.com/docs/install/vite/)
- [daisyUI themes](https://daisyui.com/docs/themes/)
