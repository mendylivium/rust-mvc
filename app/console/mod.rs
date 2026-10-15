use clap::{Parser, Subcommand};
use std::{fs::OpenOptions, io::Write};

#[derive(Parser)]
#[command(
    name = "forge",
    version,
    about = "Laravel-inspired tooling for RustMVC"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}
#[derive(Subcommand)]
pub enum Command {
    /// Run the HTTP server (run migrations first).
    Serve,
    /// Apply pending versioned migrations.
    Migrate {
        #[arg(long)]
        seed: bool,
    },
    /// Create the configured administrator without overwriting existing accounts.
    #[command(name = "db:seed")]
    DbSeed,
    /// Generate a controller using a snake_case name.
    #[command(name = "make:controller")]
    MakeController { name: String },
    /// Generate a model using a snake_case name.
    #[command(name = "make:model")]
    MakeModel { name: String },
}
pub fn generate(name: &str, controller: bool) -> anyhow::Result<()> {
    anyhow::ensure!(
        !name.is_empty()
            && name.as_bytes()[0].is_ascii_lowercase()
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
        "Use a snake_case identifier starting with a lowercase letter."
    );
    // syn parses identifiers independently of edition; `gen` is reserved in Rust 2024.
    anyhow::ensure!(
        name != "gen" && syn::parse_str::<syn::Ident>(name).is_ok(),
        "Choose a non-keyword Rust identifier."
    );
    let directory = if controller {
        "app/http/controllers"
    } else {
        "app/models"
    };
    let path = format!("{directory}/{name}.rs");
    let content = if controller {
        "use axum::Json;\nuse serde_json::{json, Value};\n\npub async fn index() -> Json<Value> {\n    Json(json!({\"data\": []}))\n}\n".to_owned()
    } else {
        let class = name
            .split('_')
            .filter(|s| !s.is_empty())
            .map(|s| format!("{}{}", s[..1].to_uppercase(), &s[1..]))
            .collect::<String>();
        anyhow::ensure!(
            syn::parse_str::<syn::Ident>(&class).is_ok(),
            "The generated type name is a reserved Rust identifier."
        );
        format!(
            "use serde::Serialize;\nuse sqlx::FromRow;\n\n#[derive(Debug, Serialize, FromRow)]\npub struct {class} {{\n    pub id: i64,\n}}\n"
        )
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    file.write_all(content.as_bytes())?;
    println!(
        "Created {path}. Add `pub mod {name};` to {directory}/mod.rs, then wire its route or migration."
    );
    Ok(())
}
