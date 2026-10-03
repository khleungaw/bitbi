use std::env;
use std::fs;
use std::path::Path;

fn main()
{
    // Tell Cargo to re-run this script if the config file changes
    println!("cargo:rerun-if-changed=config/");

    // Get the directory where Cargo puts built binaries
    let out_dir = env::var("PROFILE").unwrap();
    let target_dir = Path::new("target").join(out_dir);
    println!("target_dir: {}", target_dir.to_str().unwrap());

    // Create the destination config folder if it doesn't exist
    let dest_config_dir = target_dir.join("settings");
    fs::create_dir_all(&dest_config_dir).expect("Failed to create directory");

    // Copy your config file(s)
    // Replace "config/default.toml" with your actual file path
    fs::copy(
        "src/settings/settings.toml",
        dest_config_dir.join("settings.toml"),
    )
    .expect("Failed to copy config file to target directory");

    fs::copy(
        "src/settings/settings_local.toml",
        dest_config_dir.join("settings_local.toml"),
    )
    .expect("Failed to copy config file to target directory");
}
