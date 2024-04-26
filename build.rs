fn main() {
    println!("cargo:rerun-if-changed=stdlib/src");

    // Check that the targets in stdlib exist
    // If not, add empty files so that the build script doesn't fail
    populate_stdlib("stdlib/math.bin");
    populate_stdlib("stdlib/system.bin");
}

// Check that a file exists, and if it doesn't, create it
fn populate_stdlib(path: &str) {
    if !std::path::Path::new(path).exists() {
        // Put a blank file so that the build script doesn't fail
        std::fs::write(path, "").unwrap();
    }
}
