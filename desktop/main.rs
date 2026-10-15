#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use rust_mvc::{
    AppState,
    config::Config,
    database,
    desktop::{ApiRequest, ApiResponse, DesktopBackend},
};
use tauri::Manager;

#[tauri::command]
async fn desktop_request(
    backend: tauri::State<'_, DesktopBackend>,
    request: ApiRequest,
) -> Result<ApiResponse, String> {
    backend.request(request).await
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(directory.join("storage"))?;
            // Ignore the launcher's working directory. Configuration and data belong to this user.
            std::env::set_current_dir(&directory)?;
            let config = Config::from_env()?;
            let db = tauri::async_runtime::block_on(async {
                let db = database::connect(&config.database_url).await?;
                database::migrate(&db).await?;
                Ok::<_, anyhow::Error>(db)
            })?;
            app.manage(DesktopBackend::new(AppState { db, config, security: Default::default() }));
            tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                .title("RustMVC")
                .inner_size(1280.0, 850.0)
                .min_inner_size(800.0, 600.0)
                .on_navigation(|url| (url.scheme() == "tauri" && url.host_str() == Some("localhost")) || (matches!(url.scheme(), "http" | "https") && url.host_str() == Some("tauri.localhost") && url.port().is_none()))
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![desktop_request])
        .build(tauri::generate_context!())
        .expect("Cannot initialize RustMVC desktop. Check the application data directory and database configuration.")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                tauri::async_runtime::block_on(app.state::<DesktopBackend>().db.close());
            }
        });
}
