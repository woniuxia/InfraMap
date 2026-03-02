fn main() {
    // Rebuild the binary when icon assets change so tauri dev picks up new app icons.
    println!("cargo:rerun-if-changed=icons");
    tauri_build::build()
}
