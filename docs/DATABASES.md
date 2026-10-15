# SQLite, MySQL, and PostgreSQL

Choose the database in `.env`, then run the same Forge commands. The application includes all three SQLx drivers; changing databases does not require rebuilding with different Cargo features.

## SQLite (default)

```dotenv
DB_CONNECTION=sqlite
DB_DATABASE=storage/database.sqlite
```

SQLite needs no server. Its parent directory must exist and be writable; the starter creates `storage/` automatically. The Docker app mounts persistent storage at `/app/storage` and runs from `/app`, so the default relative path works inside and outside Docker.

## MySQL

Use MySQL 8.4 LTS. Create a database and a user with permissions for migrations and application tables before starting the app.

```dotenv
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=rustmvc
DB_USERNAME=rustmvc
DB_PASSWORD='your-database-password'
DB_SSLMODE=preferred
```

## PostgreSQL

Use PostgreSQL 17 or newer. Create the database and grant its application user permission to create and modify tables in the target schema.

```dotenv
DB_CONNECTION=pgsql
DB_HOST=127.0.0.1
DB_PORT=5432
DB_DATABASE=rustmvc
DB_USERNAME=rustmvc
DB_PASSWORD='your-database-password'
DB_SSLMODE=prefer
```

`postgres` and `postgresql` are also accepted aliases for `pgsql`. Leave `DB_PORT` empty to use 3306 for MySQL or 5432 for PostgreSQL. Credentials are URL-encoded by the configuration layer; you do not need to manually URL-encode `DB_PASSWORD`. Quote dotenv values containing `#`, spaces, or `$` so the dotenv parser/Compose preserves them.

For remote production databases, configure TLS according to your provider: PostgreSQL supports `DB_SSLMODE=verify-full`; MySQL supports `DB_SSLMODE=verify_identity`. Local Compose database images below do not provision certificates. `preferred`/`prefer` can fall back to an unencrypted connection and are intended here for local setup. For provider-specific options, use an explicit URL with the provider's CA/certificate parameters.

## Apply configuration and migrations

Remove or comment out an old `DATABASE_URL` when switching to `DB_*`. A nonempty `DATABASE_URL` takes precedence, preserving compatibility with hosting platforms, older installs, and test tooling.

```sh
./forge migrate
# First admin only; set SEED_ADMIN_EMAIL and SEED_ADMIN_PASSWORD in .env.
./forge db:seed
./forge serve
```

Restart the backend after editing `.env`. For Docker, recreate the container with `docker compose up -d`; `restart` alone does not reload the container environment.

Changing `DB_CONNECTION` connects to a different database. It **does not copy existing users or data**. Migrate/seed the selected database, or arrange a separate data transfer when moving an existing deployment. Never run database tests against a production server.

## Docker Compose database services

For an existing external database, use the base `docker-compose.yml` and set `DB_HOST` to a hostname reachable from the container. Inside a container, `127.0.0.1` points to the container itself, not the Docker host.

For bundled database services, choose one of these overlays. All commands must use the same pair of files so they refer to the same services. Database ports are not published to the host.

MySQL: use the MySQL `.env` example above and additionally set a separate, strong `DB_ROOT_PASSWORD`. The overlay sets the app's `DB_HOST=mysql`.

```sh
docker compose -f docker-compose.yml -f docker-compose.mysql.yml up -d --build --wait --wait-timeout 180
```

PostgreSQL: use the PostgreSQL `.env` example above. The overlay sets the app's `DB_HOST=postgres`.

```sh
docker compose -f docker-compose.yml -f docker-compose.pgsql.yml up -d --build --wait --wait-timeout 180
```

You can put the file selection in `.env` too, so subsequent commands stay `docker compose ...`:

```dotenv
# Linux/macOS MySQL example (use pgsql instead for PostgreSQL):
COMPOSE_FILE=docker-compose.yml:docker-compose.mysql.yml
# Windows uses a semicolon instead of a colon between paths.
```

Set `DB_CONNECTION` to match the selected overlay. The app waits for the database service health check, then applies migrations before serving. Seed the first admin using the [Docker tutorial](DOCKER.md#3-create-the-first-administrator), with the same overlay flags or `COMPOSE_FILE`. Neither service initializes application users; Forge seeding handles those.

The database images initialize their database names/users/passwords only on an empty volume. Changing `.env` later does not change an existing database account password; change the database account explicitly and then update the app configuration. Named volumes `mysql-data` and `postgres-data` persist through container recreation. `docker compose down -v` deletes them.

The bundled PostgreSQL image uses its bootstrap user for the application. For hardened production deployments, provision a separate least-privilege application account after setup, or use a managed database and the base Compose file.

Use `mysqldump` or `pg_dump` (and their matching restore tools) for server database backups; the SQLite file-copy procedure in DOCKER.md applies only to SQLite. Back up before applying migrations. Changing the backend does not make process-local login budgets shared between replicas; review the deployment limits in README.md.

## Extending the schema and writing queries

Migrations are selected by the connected driver's URL:

- SQLite: `database/migrations/*.sql` (existing migration checksums are preserved).
- MySQL: `database/migrations/mysql/*.sql`.
- PostgreSQL: `database/migrations/pgsql/*.sql`.

Add equivalent migrations with the same unique numeric version in all three locations for a feature supported on every database. Never edit an applied migration. MySQL DDL is not generally transactional; after a failed partial migration, inspect the schema before retrying.

The shared pool is `sqlx::AnyPool`. `database::sql(&pool, statement)` translates the starter's static `?` placeholders to PostgreSQL `$1`, `$2`, etc., and the internal epoch-clock expression to the driver's syntax. Use it for parameterized queries and keep passing values through `.bind(...)`. This helper is not a general SQL transpiler: use portable SQL or explicit driver branches for engine-specific operations. It accepts application-owned static statements, not arbitrary SQL submitted by users.

Example:

```rust
let user = sqlx::query_as::<_, User>(&crate::database::sql(
    &state.db,
    "SELECT * FROM users WHERE id = ?",
))
.bind(id)
.fetch_optional(&state.db)
.await?;
```

The starter keeps UTC timestamps as ISO text across drivers to maintain the same API contract. Authentication operations take the `auth_lock` row inside a transaction before changing bounded session/login storage or credentials. Preserve that lock ordering when extending authentication so password resets cannot race with session issuance.

The Projects walkthrough in [TUTORIAL.md](TUTORIAL.md) is a SQLite exercise. Its `AUTOINCREMENT` migration and `INSERT ... RETURNING` are not portable to MySQL. To port it, provide driver-specific migrations, wrap bound queries with `database::sql`, and fetch the inserted row inside the same transaction using a unique key (see `user_controller::store`).

## Backend verification

Normal `cargo test` runs with isolated SQLite databases. The HTTP suite also accepts `TEST_DATABASE_URL`, pointing to a **disposable test server** and an existing administrative database:

```sh
TEST_DATABASE_URL='postgres://testuser:testpassword@127.0.0.1:5432/postgres' cargo test --test http
TEST_DATABASE_URL='mysql://root:testpassword@127.0.0.1:3306/mysql' cargo test --test http
```

The test account must have `CREATE DATABASE` permission. Every test creates a unique `rustmvc_test_*` database. Dispose of the test server/volume afterward; these databases are intentionally not dropped automatically while connection pools may still be active. The suite verifies the same authentication, permissions, CRUD, throttling, and password-reset/session rules on each engine.

SQLx 0.8's MySQL dependency activates the RSA crate covered by RustSec RUSTSEC-2023-0071. It is no longer merely an unused lockfile entry when MySQL support is compiled. No advisory suppression is configured; review this dependency limitation for production use and keep database transport protected.

References: [SQLx 0.8 Any drivers](https://docs.rs/sqlx/0.8.6/sqlx/any/index.html), [MySQL connection options](https://docs.rs/sqlx/0.8.6/sqlx/mysql/struct.MySqlConnectOptions.html), and [PostgreSQL connection options](https://docs.rs/sqlx/0.8.6/sqlx/postgres/struct.PgConnectOptions.html).
