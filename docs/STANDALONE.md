# Standalone server and desktop builds

RustMVC has two build modes using the same Vue interface and Rust business logic.

| Mode    | Runs as                                 | User interface                                        | Transport                            |
| ------- | --------------------------------------- | ----------------------------------------------------- | ------------------------------------ |
| Server  | `rustmvc-server` / `rustmvc-server.exe` | Any supported browser; suitable for VPS or a local PC | HTTP API                             |
| Desktop | `RustMVC` / `RustMVC.exe`               | Native application window containing the OS webview   | Internal Tauri IPC; no HTTP listener |

Both modes embed the built frontend. You do not distribute Node, Cargo, source code, or a separate `dist/` directory. The database and configuration stay outside the executable so updates preserve data. “Standalone” does not mean statically linked to every OS library: the desktop still needs the OS webview, and Linux executables need compatible system libraries.

## Build commands

Install Node/npm and stable Rust, then run `npm ci`. Build on the target operating system and CPU architecture.

```sh
# Browser/server binary for the current machine (default).
make build

# Desktop window, on Linux x64.
make build MODE=desktop TARGET=linux ARCH=x64

# Server binary, on Windows x64 (if Make is installed).
make build MODE=server TARGET=windows ARCH=x64

# No Make required; these work in PowerShell or a terminal too.
npm run build:desktop -- --target windows --arch x64
npm run build:server -- --target linux --arch x64
```

Equivalent direct command:

```sh
node scripts/build.mjs --desktop --target linux --arch x64
node scripts/build.mjs --server --target linux --arch x64
```

Use `--debug` for faster local verification, or `--help` for usage. `TARGET` supports `windows`, `linux`, and `macos`; `ARCH` supports `x64` and `arm64`. The Make form uses variables because `make build --desktop` would be interpreted as a Make option. The Node build script uses the flag-style form.

Outputs:

```text
releases/server/linux-x64/rustmvc-server
releases/desktop/linux-x64/RustMVC
releases/server/windows-x64/rustmvc-server.exe
releases/desktop/windows-x64/RustMVC.exe
```

Each output folder includes `.env.example`, this guide, and `build-info.json`. Copy the executable to its destination; copy the example environment only if you need configuration. Do not overwrite an existing `.env` or database on upgrade.

The build script rejects mismatched OS/architecture instead of claiming a Windows executable was produced by a normal Linux build. Use a matching machine or the repository's **Standalone builds** GitHub Actions workflow, which builds Windows/Linux artifacts on native runners. An ARM build needs an ARM runner/machine. macOS support follows Tauri's native prerequisites; Windows/Linux are the included CI targets.

## Server mode: VPS or browser on a desktop PC

In the directory containing the server executable:

```sh
cp .env.example .env
# Set your database settings and private seed credentials in .env.
./rustmvc-server migrate --seed
./rustmvc-server serve
```

On Windows, use `./rustmvc-server.exe`. Open `http://127.0.0.1:3000` with the default configuration. The server does not open a GUI or launch a browser automatically. Existing Forge commands remain available, including `migrate` and `db:seed`; running the binary without a subcommand starts the server.

For a VPS, set `APP_HOST=0.0.0.0` if needed, configure a service manager such as systemd, and set its working directory to your deployment directory. Keep `.env` private, persist `storage/` for SQLite, and use an HTTPS reverse proxy with `COOKIE_SECURE=true`. Database selection and deployment constraints are described in `docs/DATABASES.md` and `docs/DOCKER.md` in the source repository.

The `embedded-assets` Cargo feature embeds `dist/` during compilation. Development `./forge serve` still reads `dist/` from disk unless built with that feature. Rebuild the binary whenever you change frontend assets. Register new SPA document paths in both branches of `routes/web.rs` and in Vue Router.

## Desktop mode: first launch

Run `RustMVC` (Linux/macOS) or double-click `RustMVC.exe` (Windows). A native window opens with the embedded Vue app. The first launch creates the per-user application directory, applies database migrations, and shows **Create your workspace** if the database has no users. Enter your name, email, and a password of 12–128 bytes to create the first administrator. Later launches show the normal sign-in form.

There is no default admin password and no need to run a seeding command for a new desktop workspace. If you point the desktop at an already populated database, use an existing account. The first-admin operation is available only through desktop IPC and refuses to run once users exist; the server HTTP API does not expose it.

Desktop data and `.env` live in the OS application-data directory for `com.rustmvc.desktop`:

- Linux: `$XDG_DATA_HOME/com.rustmvc.desktop`, normally `~/.local/share/com.rustmvc.desktop`.
- Windows: `%APPDATA%\com.rustmvc.desktop`.
- macOS: `~/Library/Application Support/com.rustmvc.desktop`.

The default SQLite database is `storage/database.sqlite` inside that directory. To change databases, put `.env` there and restart the application. The launcher's working directory is ignored. Process environment variables take precedence over dotenv values, so start with a clean environment if launching from a development shell with exported database overrides.

The desktop uses the same SQLite/MySQL/PostgreSQL configuration and validation as the server. Remote database credentials live on the user's computer; distribute them only when direct database access is appropriate for your application. The GUI can operate entirely locally with SQLite. Switching a database does not transfer existing data.

## Desktop architecture and platform prerequisites

The Vue API helper chooses HTTP in a browser and Tauri `invoke` inside the desktop window. The IPC command dispatches requests into the existing Axum router **in memory**. Session cookies stay in Rust memory; CSRF validation and authorization still run. Desktop navigation uses hash routes so reloads remain within the embedded app. Closing the application ends its session transport; the database persists.

The desktop process never binds an HTTP port. Opening `localhost` in a browser cannot access this desktop instance. The server executable is a separate build mode and retains normal HTTP behavior. External pages are blocked from navigating the main webview, and no shell/filesystem commands are exposed to the frontend.

Windows build prerequisites: MSVC Rust toolchain, Microsoft C++ Build Tools, and WebView2. Windows users need the WebView2 runtime (normally present on current Windows; provision it on machines where it is missing). This build emits a raw `.exe`, not an installer that installs WebView2.

Linux build prerequisites on Debian/Ubuntu:

```sh
sudo apt install build-essential pkg-config libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev
```

Linux users need compatible GTK 3 and WebKitGTK 4.1 runtime libraries and a graphical desktop session. Build on the oldest supported distribution to avoid newer glibc requirements. A raw Linux binary is not an AppImage or statically bundled webview.

macOS requires Xcode Command Line Tools and the native Rust toolchain; WKWebView is supplied by the OS. This script emits a raw binary, not a signed `.app`/DMG. Public distribution signing, notarization, installers, and auto-updates are separate packaging work.

See the official [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) and [Tauri IPC commands](https://v2.tauri.app/develop/calling-rust/) for platform details.

## Verification

```sh
cargo test --locked
npm run build
cargo test --locked --features embedded-assets
npm run test:e2e
# With desktop system prerequisites installed:
cargo check --locked --features desktop --bin rustmvc-desktop
```

The desktop transport tests exercise first-run setup, validation, duplicate-setup rejection, authentication, CSRF, profile changes, and logout without a TCP listener. A separate test ensures desktop setup is unavailable through the server router. Browser tests cover the HTTP app. Native GUI verification additionally requires launching the desktop executable on each target OS.
