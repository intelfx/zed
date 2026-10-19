#![allow(clippy::disallowed_methods, reason = "build scripts are exempt")]
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    // When a crate in zed is used as a dependency that uses the `crane` nix
    // library, it vendors each crate separately and builds it in isolation,
    // which makes `../zed/RELEASE_CHANNEL` inaccessible.
    //
    // Check if `$ZED_RELEASE_CHANNEL` is set, and if it is, use it instead of
    // the sibling crate file.
    println!("cargo::rerun-if-env-changed=ZED_RELEASE_CHANNEL");
    let release_channel = match std::env::var("ZED_RELEASE_CHANNEL").ok() {
        Some(release_channel) => release_channel,
        None => {
            let path = crate_dir.join("../zed/RELEASE_CHANNEL");
            println!("cargo:rerun-if-changed={}", path.display());
            std::fs::read_to_string(path)
                .expect("failed to read crates/zed/RELEASE_CHANNEL; set $ZED_RELEASE_CHANNEL")
                .trim()
                .to_string()
        }
    };
    println!("cargo::rustc-env=ZED_RELEASE_CHANNEL={release_channel}");

    // Likewise for `$ZED_PKG_VERSION` and `../zed/Cargo.toml`.
    println!("cargo::rerun-if-env-changed=ZED_PKG_VERSION");
    let version = match std::env::var("ZED_PKG_VERSION").ok() {
        Some(version) => version,
        None => {
            let path = crate_dir.join("../zed/Cargo.toml");
            println!("cargo::rerun-if-changed={}", path.display());
            let manifest = std::fs::read_to_string(path)
                .expect("failed to read crates/zed/Cargo.toml; set $ZED_PKG_VERSION");
            let zed_cargo_toml = toml_edit::ImDocument::parse(manifest)
                .expect("failed to parse crates/zed/Cargo.toml");
            zed_cargo_toml
                .get("package")
                .and_then(|package| package.get("version"))
                .and_then(toml_edit::Item::as_str)
                .expect("zed Cargo.toml must declare a package version string")
                .to_string()
        }
    };
    println!("cargo:rustc-env=ZED_PKG_VERSION={version}");

    // Populate git sha environment variable if git is available
    println!("cargo::rerun-if-env-changed=GITHUB_RUN_NUMBER");
    println!("cargo::rerun-if-env-changed=ZED_COMMIT_SHA");
    println!("cargo:rerun-if-changed=../../.git/logs/HEAD");
    let git_sha = match std::env::var("ZED_COMMIT_SHA").ok() {
        Some(git_sha) => {
            // In deterministic build environments such as Nix, we inject the commit sha into the build script.
            Some(git_sha)
        }
        None => {
            if let Some(output) = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .output()
                .ok()
                && output.status.success()
            {
                let git_sha = String::from_utf8_lossy(&output.stdout);
                Some(git_sha.trim().to_string())
            } else {
                None
            }
        }
    };

    if let Some(git_sha) = git_sha {
        println!("cargo:rustc-env=ZED_COMMIT_SHA={git_sha}");

        if let Ok(build_identifier) = std::env::var("GITHUB_RUN_NUMBER") {
            println!("cargo:rustc-env=ZED_BUILD_ID={build_identifier}");
        }

        if let Ok(build_profile) = std::env::var("PROFILE")
            && build_profile == "release"
        {
            // This is currently the best way to make `cargo build ...`'s build script
            // to print something to stdout without extra verbosity.
            println!("cargo::warning=Info: using '{git_sha}' hash for ZED_COMMIT_SHA env var");
        }
    }
}
