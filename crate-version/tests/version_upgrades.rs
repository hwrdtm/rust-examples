use std::{fs, process::Command};

#[test]
fn test_ugprade_version() {
    build_crate();
    run_crate();

    let crate_version_handle = update_node_crate_version("3.1.2".to_string());
    let cargo_toml = fs::read_to_string("./Cargo.toml").expect("Failed to read Cargo.toml");
    println!("Updated Cargo.toml: {}", cargo_toml);

    build_crate();
    run_crate();
}

fn run_crate() {
    Command::new("../target/debug/crate-version")
        .spawn()
        .expect("Failed to launch crate");
}

fn build_crate() {
    // First, clear the executable at ./target/debug/crate_version
    let _ = std::fs::remove_file("../target/debug/crate-version")
        .expect("Failed to remove crate_version executable");
    println!("Removed old crate_version executable");

    let start = std::time::Instant::now();
    let _build_command = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("Failed to compile with cargo build");

    println!(
        "Building took {} ms.  Starting crate.  Env: {:?}",
        start.elapsed().as_millis(),
        std::env::current_dir()
    );
}

fn update_node_crate_version(new_crate_version: String) -> CrateVersionHandle {
    // First get the current crate version. Do this by running the command `cargo pkgid` and parsing
    // the characters at the end after the `@`.
    let cmd = Command::new("cargo")
        .args(["pkgid"])
        .output()
        .expect("Failed to get current crate version");
    assert!(cmd.status.success());
    let current_crate_version = String::from_utf8(cmd.stdout)
        .unwrap()
        .split("#")
        .last()
        .unwrap()
        .trim()
        .to_string();
    println!(
        "Current crate version before updating: {:?}",
        current_crate_version
    );

    // Update the crate version.
    let cargo_toml = fs::read_to_string("./Cargo.toml").expect("Failed to read Cargo.toml");
    let mut doc = cargo_toml
        .parse::<toml_edit::Document>()
        .expect("Failed to parse Cargo.toml");
    doc["package"]["version"] = toml_edit::value(new_crate_version.clone());
    fs::write("./Cargo.toml", doc.to_string()).expect("Failed to write Cargo.toml");
    println!("Updated node crate version to {}", new_crate_version);

    CrateVersionHandle(current_crate_version)
}

/// A simple struct that stores a version, and updates the crate to this version when this
/// struct is dropped.
struct CrateVersionHandle(String);

impl Drop for CrateVersionHandle {
    fn drop(&mut self) {
        println!("Reverting crate version back to {:?}", self.0);

        // Update the crate version.
        let cargo_toml = fs::read_to_string("./Cargo.toml").expect("Failed to read Cargo.toml");
        let mut doc = cargo_toml
            .parse::<toml_edit::Document>()
            .expect("Failed to parse Cargo.toml");
        doc["package"]["version"] = toml_edit::value(self.0.clone());
        println!("Updated node crate version to {}", self.0);
    }
}
