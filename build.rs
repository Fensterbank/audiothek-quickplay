use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=podcasts.json");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let profile = env::var("PROFILE").unwrap();
    let source_path = manifest_dir.join("podcasts.json");

    let mut target_path = manifest_dir.join("target");
    if let Ok(target) = env::var("TARGET") {
        if !target.is_empty() {
            target_path = target_path.join(target);
        }
    }
    target_path = target_path.join(profile);

    // Ensure the target directory exists
    fs::create_dir_all(&target_path).expect("Failed to create target directory");

    let dest_path = target_path.join("podcasts.json");
    fs::copy(&source_path, &dest_path).unwrap_or_else(|e| panic!("Failed to copy podcasts.json from {:?} to {:?}: {}", source_path, dest_path, e));

    // For Windows cross-compile, the deps are in a different folder
    if let Ok(target) = env::var("TARGET") {
        if target.contains("windows") {
            let deps_path = target_path.join("deps");
            fs::create_dir_all(&deps_path).ok(); // Ensure deps directory exists
            fs::copy(&source_path, deps_path.join("podcasts.json")).ok(); // Ignore error if deps doesn't exist
        }
    }
}
