use std::{fs, process::Command};

fn fixture() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("rustmvc-cli-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(path.join("app/models")).unwrap();
    fs::create_dir_all(path.join("app/http/controllers")).unwrap();
    path
}
#[test]
fn generators_reject_reserved_and_transformed_identifiers_without_writing() {
    let dir = fixture();
    for name in ["self_", "self__", "do", "abstract", "gen", "../escape"] {
        let output = Command::new(env!("CARGO_BIN_EXE_forge"))
            .current_dir(&dir)
            .args(["make:model", name])
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted invalid name {name}");
    }
    assert_eq!(fs::read_dir(dir.join("app/models")).unwrap().count(), 0);
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(&dir)
        .args(["make:model", "order_item"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let source = fs::read_to_string(dir.join("app/models/order_item.rs")).unwrap();
    assert!(source.contains("struct OrderItem"));
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(&dir)
        .args(["make:model", "order_item"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(dir.join("app/models/order_item.rs")).unwrap(),
        source
    );
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn seeder_validates_email_and_keeps_existing_credentials() {
    let dir = fixture();
    let database = format!("sqlite://{}", dir.join("seed.sqlite").display());
    let run = |email: &str, password: &str| {
        Command::new(env!("CARGO_BIN_EXE_forge"))
            .current_dir(&dir)
            .env("DATABASE_URL", &database)
            .env("SEED_ADMIN_EMAIL", email)
            .env("SEED_ADMIN_PASSWORD", password)
            .args(["migrate", "--seed"])
            .output()
            .unwrap()
    };
    for email in ["", "not-an-email"] {
        let result = run(email, "test-password-123");
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("SEED_ADMIN_EMAIL"));
    }
    assert!(
        run(" Valid@Example.com ", "test-password-123")
            .status
            .success()
    );
    let rt = tokio::runtime::Runtime::new().unwrap();
    let before: String = rt.block_on(async {
        let db = sqlx::SqlitePool::connect(&database).await.unwrap();
        sqlx::query_scalar("SELECT password FROM users WHERE email = 'valid@example.com'")
            .fetch_one(&db)
            .await
            .unwrap()
    });
    assert!(
        run("valid@example.com", "different-password-123")
            .status
            .success()
    );
    let after: String = rt.block_on(async {
        let db = sqlx::SqlitePool::connect(&database).await.unwrap();
        sqlx::query_scalar("SELECT password FROM users WHERE email = 'valid@example.com'")
            .fetch_one(&db)
            .await
            .unwrap()
    });
    assert_eq!(before, after);
    fs::remove_dir_all(dir).unwrap();
}
