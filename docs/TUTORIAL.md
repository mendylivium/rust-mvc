# RustMVC tutorial

A complete path from first run to your first database-backed page. Commands run from the repository root. The included UI examples are ready to browse; the Projects feature below is an exercise you add yourself, not an already-installed module.

## 1. Start the application

Install stable Rust with Cargo, a C compiler/linker, and Node.js 22.12 or newer. Check `cargo --version`, `node --version`, and `npm --version`.

```sh
cp .env.example .env
npm ci
cargo fetch
```

In `.env`, set `SEED_ADMIN_EMAIL` and a private `SEED_ADMIN_PASSWORD` of 12–128 bytes. Keep `COOKIE_SECURE=false` for local HTTP. The SQLite database defaults to `storage/database.sqlite`; no separate database server is needed. Never commit `.env` or database files.

```sh
./forge migrate --seed
node scripts/dev.mjs
```

On Windows, use `forge.cmd` instead of `./forge`. Alternatively, run `./forge serve` and `npm run dev` in separate terminals. Open **http://127.0.0.1:5173**, then log in with your seed credentials. Vite forwards `/api` requests to Rust on port 3000. Keep the hostname consistent so cookies work.

Expected result: the dashboard shows your real database totals, initially one administrator. Seeding again does not reset the existing password. Vue changes update through HMR; restart the Rust process after backend changes.

## 2. Explore the included pages

| Page               | Route                | What to learn                                                                   |
| ------------------ | -------------------- | ------------------------------------------------------------------------------- |
| Overview           | `/`                  | API statistics, loading/error states, registration chart                        |
| Users              | `/users`             | Persistent CRUD, search, role filtering, pagination, confirmation dialogs       |
| Settings           | `/settings`          | Persistent profile form and field errors                                        |
| Getting started    | `/guide`             | In-app orientation and commands                                                 |
| Elements & buttons | `/examples/elements` | Button variants, badges, alerts, dialog, progress, disclosure, content switcher |
| Cards              | `/examples/cards`    | Metrics, project, profile, feature, and empty-state cards                       |
| Tables             | `/examples/tables`   | Local search, status filter, sorting, selection, pagination, empty results      |
| Forms              | `/examples/forms`    | Inputs, select, textarea, radios, toggle, range, validation, reset              |

Examples use clearly labelled sample data. Their interactions reset when you leave/reload; they do not modify users or your database. Only administrators can access Overview and Users. Every signed-in user can view the examples and Settings.

Try adding a member through Users, signing out, and signing in as that member. The member can edit their own name but cannot use the administrator API. Vue guards improve navigation; Rust policies enforce the actual permissions.

## 3. Understand the request flow

```text
Vue page → lib/api.js → /api route → session/CSRF middleware
         → AuthUser → policy → validated request → SQLx → JSON
         → Vue state → rendered UI
```

- `bootstrap/application.rs` assembles the server and shared state.
- `routes/api.rs` declares API routes. `routes/web.rs` serves built frontend routes.
- `app/http/controllers` coordinates handlers; `app/models` defines database rows.
- `app/http/requests` validates input; its custom extractors preserve JSON error responses.
- `app/policies` controls access. Never rely only on a hidden button or frontend route guard.
- `resources/js/lib/api.js` supplies JSON encoding, same-origin cookies, and the current CSRF token.

The router loads `/api/auth/session` before protected navigation. Writes need the session cookie and `X-CSRF-Token`. Use `api('/path', { method: 'POST', body: object })`; do not manually stringify the body or put session credentials in localStorage. Errors expose `message`, `status`, and field `errors`. A CSRF failure refreshes session state but does not automatically replay a write.

## 4. Build a Projects feature

This exercise targets SQLite. The starter itself also supports MySQL/PostgreSQL through `.env`; see [DATABASES.md](DATABASES.md) for configuration and portability conventions.

This exercise implements an administrator-only project list and create form with persistent SQLite storage. It intentionally starts with two operations; use Users as the reference for update/delete and server pagination.

### Create the migration

Create `database/migrations/20261016000001_create_projects.sql`. Use a larger unique numeric prefix if your repository already has a later migration.

```sql
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(name) BETWEEN 2 AND 80),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
```

```sh
./forge migrate
./forge make:model project
./forge make:controller project_controller
```

Generators refuse to overwrite existing files. Replace their generated contents with the code below and register both modules. Applied migrations should stay unchanged; make a new migration for subsequent schema changes.

### Define the model

`app/models/project.rs`:

```rust
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}
```

Append `pub mod project;` to `app/models/mod.rs`.

### Validate input

Create `app/http/requests/project_request.rs`:

```rust
use crate::app::http::error::{AppError, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ProjectRequest {
    pub name: String,
}

impl ProjectRequest {
    pub fn validated(mut self) -> Result<Self> {
        self.name = self.name.trim().to_owned();
        if !(2..=80).contains(&self.name.chars().count()) {
            return Err(AppError::validation(
                "name",
                "Use a project name between 2 and 80 characters.",
            ));
        }
        Ok(self)
    }
}
```

Append `pub mod project_request;` to `app/http/requests/mod.rs`. Browser validation helps users, but server validation is required for every write.

### Implement the controller

`app/http/controllers/project_controller.rs`:

```rust
use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};
use crate::{
    AppState,
    app::{
        http::{
            error::Result,
            middleware::session::AuthUser,
            requests::{extractors::Json as Input, project_request::ProjectRequest},
        },
        models::project::Project,
        policies,
    },
};

pub async fn index(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
) -> Result<Json<Value>> {
    policies::admin(&actor)?;
    let projects: Vec<Project> = sqlx::query_as(
        "SELECT id, name, created_at FROM projects ORDER BY id DESC LIMIT 100",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(json!({ "data": projects })))
}

pub async fn store(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
    Input(input): Input<ProjectRequest>,
) -> Result<(StatusCode, Json<Value>)> {
    policies::admin(&actor)?;
    let input = input.validated()?;
    let project: Project = sqlx::query_as(
        "INSERT INTO projects (name) VALUES (?) RETURNING id, name, created_at",
    )
    .bind(input.name)
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(json!({ "data": project }))))
}
```

Append `pub mod project_controller;` to `app/http/controllers/mod.rs`. Values use `.bind()` rather than SQL string interpolation. The list returns the latest 100 projects; add server pagination before using it for a large collection.

### Register the API

Add an import in `routes/api.rs`:

```rust
use crate::app::http::controllers::project_controller as projects;
```

Inside the existing `Router::new()` chain, before `.fallback(...)`, add:

```rust
.route("/projects", get(projects::index).post(projects::store))
```

Keep the existing CSRF middleware layer. Restart Rust. An unauthenticated request to `/api/projects` must return 401, and a member request must return 403.

### Create the Vue page

Create `resources/js/pages/ProjectsPage.vue`:

```vue
<script setup>
import { onMounted, ref } from 'vue'
import { api } from '../lib/api'
import FeedbackMessage from '../components/FeedbackMessage.vue'

const projects = ref([]),
  name = ref(''),
  loading = ref(true),
  saving = ref(false)
const error = ref(''),
  fieldError = ref(''),
  success = ref('')
async function load() {
  loading.value = true
  error.value = ''
  try {
    projects.value = (await api('/projects')).data
  } catch (e) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}
async function create() {
  if (saving.value) return
  saving.value = true
  error.value = ''
  fieldError.value = ''
  success.value = ''
  try {
    const response = await api('/projects', { method: 'POST', body: { name: name.value } })
    projects.value = [response.data, ...projects.value].slice(0, 100)
    name.value = ''
    success.value = 'Project created.'
  } catch (e) {
    error.value = e.message
    fieldError.value = e.errors?.name?.[0] || ''
  } finally {
    saving.value = false
  }
}
onMounted(load)
</script>

<template>
  <div class="page-heading">
    <div>
      <p class="eyebrow">WORKSPACE</p>
      <h1>Projects</h1>
      <p class="muted">Your latest 100 projects.</p>
    </div>
  </div>
  <FeedbackMessage :message="error" />
  <FeedbackMessage :message="success" success />
  <section class="card example-panel">
    <h2>Create a project</h2>
    <form class="example-toolbar" @submit.prevent="create">
      <label class="example-field"
        >Project name
        <input
          v-model="name"
          class="input"
          required
          minlength="2"
          maxlength="80"
          :aria-invalid="!!fieldError"
          :aria-describedby="fieldError ? 'project-error' : undefined"
        />
        <span v-if="fieldError" id="project-error" class="field-error">{{ fieldError }}</span>
      </label>
      <button class="btn btn-primary" :disabled="saving || loading">
        {{ saving ? 'Creating…' : 'Create project' }}
      </button>
    </form>
    <p v-if="loading" role="status">Loading projects…</p>
    <template v-else>
      <button v-if="error" class="btn btn-outline" @click="load">Reload projects</button>
      <div class="table-scroll">
        <table class="table">
          <caption>
            Latest projects
          </caption>
          <thead>
            <tr>
              <th scope="col">Name</th>
              <th scope="col">Created (UTC)</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="project in projects" :key="project.id">
              <th scope="row">{{ project.name }}</th>
              <td>{{ project.created_at }}</td>
            </tr>
            <tr v-if="!projects.length">
              <td colspan="2">No projects yet. Create your first one above.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </section>
</template>
```

### Register navigation and direct links

Add this entry inside the AdminLayout `children` array in `resources/js/router.js`:

```js
{
  path: 'projects',
  component: () => import('./pages/ProjectsPage.vue'),
  meta: { title: 'Projects', admin: true },
},
```

Add a link inside the navigation in `resources/js/layouts/AdminLayout.vue`:

```vue
<RouterLink
  v-if="auth.user?.role === 'admin'"
  to="/projects"
  active-class="selected"
  @click="open = false"
>
  <AppIcon name="Layers3" />Projects
</RouterLink>
```

Add a production document route before `.fallback_service(...)` in `routes/web.rs`:

```rust
.route_service("/projects", ServeFile::new("dist/index.html"))
```

Now open `/projects` as an administrator, create a project, and reload. The row should persist. Refreshing a known production page returns HTTP 200; unknown URLs still return 404.

## 5. Customize the UI

Copy markup from `resources/js/pages/examples/` into your feature. Common patterns:

```html
<button class="btn btn-primary">Save changes</button>
<span class="badge badge-success">Active</span>
<section class="card example-panel"><h2>Section title</h2></section>
<label class="example-field">Name<input class="input" /></label>
```

Use `AppDialog` for modal forms/confirmations and `FeedbackMessage` for success/errors. Give a dialog heading a unique ID and pass that ID through `labelledby`. Native dialog handles focus containment; the wrapper restores focus when closed. Keep controls labelled and table headers semantic. Avoid communicating status through color alone.

Change colors in the `rustmvc` theme in `resources/css/app.css`. Keep `--radius-selector`, `--radius-field`, and `--radius-box` at `0.25rem` or smaller. Do not add pill/circle utilities that override this limit. Check hover, focus, disabled, error, dark sidebar, and mobile states after styling changes. The showcase layout helpers are in the final section of the stylesheet; they are shared by the exercise above.

## 6. Verify the feature

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
npm run format
npm run build
npx playwright install chromium
npm run test:e2e
```

The included tests cover the starter, including the sample pages. They use an isolated temporary database, not your local workspace database. The tutorial module needs its own additional tests when you implement it.

For Projects, extend the HTTP tests in `tests/http.rs` using its `Client` and `app()` helpers. Check guest GET → 401, member GET/POST → 403, administrator GET → 200, invalid/blank name → 422 with `errors.name`, valid POST → 201, subsequent GET contains the row, and missing CSRF on POST → 403. Add a browser test that creates a project, reloads, and verifies persistence. Check direct `/projects` returns 200 in the production server, keyboard operation, narrow screens, and an API failure state.

Existing Playwright tests demonstrate login, form submission, axe accessibility checks, mobile keyboard navigation, and maximum rendered corner radius. Screenshots/traces go into ignored `test-results/`.

## 7. Build and deploy

For a binary with embedded frontend assets or a native desktop window, follow [STANDALONE.md](STANDALONE.md). Desktop uses in-process IPC and first-run setup; server mode uses HTTP.

For a container deployment without installing Rust or Node on the host, follow the [Docker Compose tutorial](DOCKER.md). It covers the included `docker-compose.yml`, persistent storage, first-admin provisioning, HTTPS, updates, and backup/restore. The commands below are for deployment without Docker.

```sh
npm ci
npm run build
cargo build --release --locked
./target/release/forge migrate
./target/release/forge serve
```

Set production environment variables before running those last two commands. Use HTTPS and `COOKIE_SECURE=true`; set `APP_HOST=0.0.0.0` when required by your reverse proxy/container. Run from a directory containing `dist/` and writable `storage/`. Deploy the binary and frontend assets together; SQL migrations are embedded in the binary. Back up the database before applying production migrations. Keep persistent storage across releases.

For the first production administrator only, supply seed credentials and run `./target/release/forge db:seed`. Remove the seed password from deployment configuration after provisioning if it is no longer needed. Never reuse the browser-test password.

The starter uses SQLite and process-local authentication budgets. See the README's Production section for actual limits and deployment boundaries, plus `AUDIT.md` for remediation and the outstanding optional dependency advisory. Queues, mail, password recovery, and distributed session infrastructure are not included.

## Troubleshooting

| Symptom                                 | Check                                                                                                                        |
| --------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `cargo` not found                       | Install Rust; open a new shell so Cargo is on PATH.                                                                          |
| Database/table error                    | Run `./forge migrate` from the repository root and check `DATABASE_URL`.                                                     |
| Login fails after reseeding             | Seeding never changes an existing password. Check the original credentials; do not delete a database containing needed data. |
| Cookie/session does not persist locally | Use `127.0.0.1` consistently and `COOKIE_SECURE=false` for local HTTP.                                                       |
| API unreachable in development          | Start Rust on port 3000; if changed, update `vite.config.js`'s proxy.                                                        |
| HTTP 429                                | Wait for the applicable throttle window; see the README for account/process limits.                                          |
| CSRF error after another tab logs in    | Submit again after the client refreshes session state. Writes are not automatically replayed.                                |
| New production page returns 404         | Register it in both Vue Router and `routes/web.rs`, rebuild both layers, and restart.                                        |
| UI changes absent in production         | Re-run `npm run build` and deploy the new `dist/`.                                                                           |
| Browser executable missing              | Run `npx playwright install chromium`; on CI/Linux install its system dependencies too.                                      |

For the repository map, complete API list, and framework reference links, return to [README.md](../README.md).
