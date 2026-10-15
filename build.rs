fn main() {
    println!("cargo:rerun-if-changed=database/migrations");
    println!("cargo:rerun-if-changed=dist");
    #[cfg(feature = "desktop")]
    tauri_build::build();
}
