fn main() {
    println!("cargo:rerun-if-changed=database/migrations");
}
