use clap::Parser;
use rust_mvc::{
    AppState,
    app::console::{Cli, Command},
    config::Config,
    database,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_mvc=info,tower_http=info".into()),
        )
        .init();
    let command = Cli::parse().command.unwrap_or(Command::Serve);
    if let Command::MakeController { name } | Command::MakeModel { name } = &command {
        return rust_mvc::app::console::generate(
            name,
            matches!(command, Command::MakeController { .. }),
        );
    }
    let db = database::connect(&config.database_url).await?;
    match command {
        Command::Migrate { seed } => {
            database::migrate(&db).await?;
            println!("Migrations complete.");
            if seed {
                database::seeders::run(&db).await?;
            }
        }
        Command::DbSeed => database::seeders::run(&db).await?,
        Command::Serve => {
            let address = format!("{}:{}", config.host, config.port);
            let listener = tokio::net::TcpListener::bind(&address).await?;
            tracing::info!(%address, "RustMVC listening");
            axum::serve(
                listener,
                rust_mvc::application::router(AppState { db, config }),
            )
            .with_graceful_shutdown(shutdown())
            .await?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

async fn shutdown() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
