# Deploy with Docker Compose

This setup embeds the built Vite/Vue frontend into the Rust server binary inside one image. Docker uses server mode; the native desktop build is documented in [STANDALONE.md](STANDALONE.md). Rust serves the UI and API on container port 3000. SQLite is the default and lives in a persistent named volume at `/app/storage`. For MySQL/PostgreSQL, see [database configuration and optional Compose services](DATABASES.md). You do not need Rust, Node, or a database server installed on the deployment host.

## 1. Prerequisites and configuration

Install Docker Engine with the Compose plugin, or Docker Desktop. Verify:

```sh
docker version
docker compose version
```

Use modern `docker compose`, not the legacy `docker-compose` executable. Run all commands below from the repository root. Keep the same checkout directory/project name between deployments so Compose reuses the same database volume.

If you do not already have a `.env`, create one:

```sh
cp .env.example .env
```

Keep your existing `.env` if you already use the starter locally. Compose reads it for variable interpolation; it passes only the environment variables explicitly listed in `docker-compose.yml` to the application. The local `storage/` directory is not mounted, so Docker and local development have separate databases.

| Variable         | Default                         | Purpose                                 |
| ---------------- | ------------------------------- | --------------------------------------- |
| `APP_NAME`       | `RustMVC`                       | Workspace application name              |
| `DOCKER_BIND_IP` | `127.0.0.1`                     | Host interface exposing the application |
| `DOCKER_PORT`    | `8080`                          | Host port; container port stays 3000    |
| `COOKIE_SECURE`  | `false`                         | Set `true` when accessed through HTTPS  |
| `RUST_LOG`       | `rust_mvc=info,tower_http=info` | Server logging                          |

Compose fixes `APP_HOST=0.0.0.0` and `APP_PORT=3000` inside the container. It forwards `DB_*` settings and the optional `DATABASE_URL` override from `.env`. For SQLite, keep `DB_DATABASE=storage/database.sqlite` so data stays in the mounted volume. Use a database name instead of a file path for MySQL/PostgreSQL.

## 2. Build and start

```sh
docker compose config --quiet
docker compose up -d --build --wait --wait-timeout 180
docker compose ps
docker compose logs --tail=100 app
```

The first image build can take several minutes. Startup applies pending migrations, then starts Rust only if they succeed. The HTTP health check uses `/api/health`; it reports server liveness, not a full database integrity check. The container runs as the unprivileged `app` user. Docker initializes a new named volume from the image's writable storage directory.

Open **http://127.0.0.1:8080** (or your configured host port). No Vite dev server is needed. You will need to create the first administrator before signing in.

## 3. Create the first administrator

In Bash, prompt for credentials without putting the password in command history:

```sh
read -r -p 'Admin email: ' SEED_ADMIN_EMAIL
read -r -s -p 'Admin password (12–128 bytes): ' SEED_ADMIN_PASSWORD
printf '\n'
export SEED_ADMIN_EMAIL SEED_ADMIN_PASSWORD
docker compose run --rm --no-deps \
  -e SEED_ADMIN_EMAIL -e SEED_ADMIN_PASSWORD app forge db:seed
unset SEED_ADMIN_EMAIL SEED_ADMIN_PASSWORD
```

In PowerShell, use the equivalent environment-variable flow:

```powershell
$env:SEED_ADMIN_EMAIL = Read-Host 'Admin email'
$securePassword = Read-Host 'Admin password (12–128 bytes)' -AsSecureString
$env:SEED_ADMIN_PASSWORD = [System.Net.NetworkCredential]::new('', $securePassword).Password
docker compose run --rm --no-deps -e SEED_ADMIN_EMAIL -e SEED_ADMIN_PASSWORD app forge db:seed
Remove-Item Env:SEED_ADMIN_EMAIL, Env:SEED_ADMIN_PASSWORD
Remove-Variable securePassword
```

This one-off container uses the same image and database volume. It overrides the normal startup command, runs the seeder, and is removed afterward. Run it after the app's initial successful startup so migrations exist. Seed credentials are passed only to this one-off container, not to the long-running application. Values placed in `.env` are not automatically exported to the shell for these commands.

Sign in with the credentials you entered. Seeding is idempotent: it does not change an existing user's password. Use the application's user management to manage accounts after initial provisioning.

## 4. Use HTTPS in production

The default binding exposes port 8080 only on the Docker host's loopback interface. A host-installed reverse proxy can forward HTTPS requests to `http://127.0.0.1:8080`. Configure your domain and TLS certificate in that proxy, forward both frontend and `/api` requests to the same application, then set:

```dotenv
COOKIE_SECURE=true
```

Apply the changed environment:

```sh
docker compose up -d --wait --wait-timeout 180
```

Use your HTTPS domain for login. Secure cookies will not work over ordinary remote HTTP. TLS and a reverse proxy are not bundled in this Compose file. If your proxy runs in another container, connect it to the Compose network and target `app:3000`; its own `127.0.0.1` is not this application.

To expose a host port directly on other interfaces, set `DOCKER_BIND_IP=0.0.0.0` and configure the host firewall appropriately. For public deployment, terminate HTTPS before application traffic reaches Rust.

Run one application replica with this SQLite setup. Authentication process budgets are not shared across replicas. See [README.md](../README.md#production) for the current limits.

## 5. Stop, restart, and update

```sh
# Restart the existing container; startup rechecks migrations.
docker compose restart app

# Stop and remove containers/network, retaining the database volume.
docker compose down

# Start again with the existing database.
docker compose up -d --wait --wait-timeout 180
```

For a code update, obtain the new source, build the new image, take a backup as described below, then recreate the app:

```sh
docker compose build --pull app
# Follow the backup procedure before applying new migrations.
docker compose up -d --force-recreate --wait --wait-timeout 180
```

Startup applies migrations embedded in the new binary. A failed migration prevents serving and appears in `docker compose logs app`. Investigate the failure instead of deleting the volume. Container restarts are managed by `unless-stopped`; health-check failure alone does not trigger a Docker restart.

Migrations are forward-only. Replacing the image with an older one does not undo database changes; restoring a backup may be needed for rollback.

## 6. Back up and restore SQLite

This section applies only to `DB_CONNECTION=sqlite`. For MySQL/PostgreSQL, use native server backup tools as described in [DATABASES.md](DATABASES.md).

Keep all database writers stopped while copying SQLite files. These commands assume the normal single app container and no concurrent seed/maintenance jobs.

Back up the entire storage directory, including any SQLite sidecar files, with Bash:

```sh
mkdir -p backups
docker compose stop app
backup_path="backups/storage-$(date -u +%Y%m%dT%H%M%SZ).tar.gz"
docker compose run --rm --no-deps -T app \
  tar -C /app/storage -czf - . > "$backup_path"
# Confirm tar succeeded and the archive is readable before relying on it.
tar -tzf "$backup_path"
docker compose up -d --wait --wait-timeout 180
```

The `backups/` directory is ignored by Git and excluded from the image build context. Copy backups to separate, access-controlled storage. A backup includes user records, password hashes, and sessions.

To restore, first make a fresh backup of the current database. Choose a known-good archive and a compatible application version. The following commands **replace all current container database files** with that archive:

```sh
restore_path=backups/storage-REPLACE-WITH-TIMESTAMP.tar.gz
tar -tzf "$restore_path"
# Continue only after the archive check succeeds.
docker compose stop app
docker compose run --rm --no-deps -T app sh -c \
  'find /app/storage -mindepth 1 -maxdepth 1 -exec rm -rf -- {} + && tar -C /app/storage -xzf -' \
  < "$restore_path"
docker compose up -d --wait --wait-timeout 180
```

Use archives produced by your own backup procedure. Confirm restore succeeds before starting the app; startup may apply migrations from the deployed image to the restored database.

**Do not run `docker compose down -v` unless you intend to permanently delete the application's named database volume.** Ordinary `down`, container recreation, and image rebuilds retain it. Changing the Compose project name creates a different volume and can make the workspace appear empty.

## 7. Troubleshooting and checks

```sh
docker compose config --quiet
docker compose ps -a
docker compose logs --tail=200 app
docker compose exec app curl --fail --silent http://127.0.0.1:3000/api/health
```

- **Port occupied:** choose another `DOCKER_PORT`, then run `docker compose up -d`.
- **Unhealthy or restarting:** inspect logs for migration/configuration errors. Check that the named volume is writable by the image's `app` user; avoid replacing it with a root-owned host bind mount.
- **Empty workspace:** confirm you are using the same Compose project and volume. The host's local development database is separate.
- **Login does not persist:** check `COOKIE_SECURE` and use one consistent host/domain.
- **Changed `.env` not applied:** recreate with `docker compose up -d`; `restart` does not replace container environment values.
- **Seed fails:** check password length and the initial migration logs. Existing users are never overwritten by the seeder.

After deployment, verify `/api/health`, `/login`, and direct `/examples/tables` navigation. Sign in, update a profile, recreate the app container, and confirm the update persists. Use the normal project tests described in [the tutorial](TUTORIAL.md#6-verify-the-feature) for broader application checks.

References: [Compose services](https://docs.docker.com/reference/compose-file/services/), [Docker volumes](https://docs.docker.com/engine/storage/volumes/), and [Docker Compose installation](https://docs.docker.com/compose/install/).
