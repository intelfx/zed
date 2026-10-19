use std::path::PathBuf;

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
}
