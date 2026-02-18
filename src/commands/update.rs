use crate::cli::UpdateArgs;
use crate::output::print_success;
use anyhow::{bail, Result};
use colored::*;
use std::env;
use std::path::PathBuf;
use std::process::Command;

const GITHUB_REPO: &str = "hongkongkiwi/elevenlabs-cli";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn execute(args: UpdateArgs) -> Result<()> {
    println!("{}", "ElevenLabs CLI Updater".bold().underline());
    println!();

    // Check for updates first
    let latest_version = fetch_latest_version().await?;
    let current = VERSION.to_string();

    println!("  Current version: {}", current.cyan());
    println!("  Latest version:  {}", latest_version.cyan());
    println!();

    if args.check {
        if current == latest_version {
            print_success("You are already on the latest version!");
        } else {
            println!(
                "{} A new version is available: {}",
                "Update available:".yellow(),
                latest_version.green()
            );
            println!("  Run `elevenlabs-cli update` to install.");
        }
        return Ok(());
    }

    if current == latest_version && !args.force {
        print_success("You are already on the latest version!");
        println!("  Use --force to reinstall.");
        return Ok(());
    }

    // Detect installation method
    let install_method = detect_install_method()?;

    match install_method {
        InstallMethod::Homebrew => update_via_homebrew()?,
        InstallMethod::Cargo => update_via_cargo()?,
        InstallMethod::Snap => update_via_snap()?,
        InstallMethod::Aur => update_via_aur()?,
        InstallMethod::Binary(path) => update_via_binary(&path, &latest_version).await?,
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
enum InstallMethod {
    Homebrew,
    Cargo,
    Snap,
    Aur,
    Binary(PathBuf),
}

fn detect_install_method() -> Result<InstallMethod> {
    let exe_path = env::current_exe()?;
    let exe_str = exe_path.to_string_lossy();

    // Check for Homebrew
    if exe_str.contains("/homebrew/") || exe_str.contains("/Cellar/") {
        return Ok(InstallMethod::Homebrew);
    }

    // Check for Linuxbrew
    if exe_str.contains("/.linuxbrew/") {
        return Ok(InstallMethod::Homebrew);
    }

    // Check for Snap
    if exe_str.contains("/snap/") {
        return Ok(InstallMethod::Snap);
    }

    // Check for Cargo (installed via cargo install)
    if exe_str.contains("/.cargo/bin/") {
        return Ok(InstallMethod::Cargo);
    }

    // Check for AUR (usually in /usr/bin on Arch)
    if exe_str.contains("/usr/bin/") && is_arch_linux() {
        return Ok(InstallMethod::Aur);
    }

    // Otherwise, treat as standalone binary
    Ok(InstallMethod::Binary(exe_path))
}

fn is_arch_linux() -> bool {
    std::path::Path::new("/etc/arch-release").exists()
}

async fn fetch_latest_version() -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::Client::builder()
        .user_agent("elevenlabs-cli")
        .build()?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch release info: {}", e))?;

    if !response.status().is_success() {
        bail!("Failed to fetch latest version from GitHub");
    }

    let json: serde_json::Value = response.json().await?;
    let tag_name = json["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid response from GitHub"))?;

    // Remove 'v' prefix if present
    Ok(tag_name.trim_start_matches('v').to_string())
}

fn update_via_homebrew() -> Result<()> {
    println!("{} Detected Homebrew installation", "ℹ".blue());
    println!("  Running: {}", "brew upgrade elevenlabs-cli".cyan());

    let status = Command::new("brew")
        .args(["upgrade", "elevenlabs-cli"])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run brew: {}", e))?;

    if status.success() {
        print_success("Successfully updated via Homebrew!");
        Ok(())
    } else {
        bail!("Failed to update via Homebrew. Try running: brew upgrade elevenlabs-cli");
    }
}

fn update_via_cargo() -> Result<()> {
    println!("{} Detected Cargo installation", "ℹ".blue());
    println!(
        "  Running: {}",
        "cargo install elevenlabs-cli --force".cyan()
    );

    let status = Command::new("cargo")
        .args(["install", "elevenlabs-cli", "--force"])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run cargo: {}", e))?;

    if status.success() {
        print_success("Successfully updated via Cargo!");
        Ok(())
    } else {
        bail!("Failed to update via Cargo. Try running: cargo install elevenlabs-cli --force");
    }
}

fn update_via_snap() -> Result<()> {
    println!("{} Detected Snap installation", "ℹ".blue());
    println!("  Running: {}", "sudo snap refresh elevenlabs-cli".cyan());

    let status = Command::new("sudo")
        .args(["snap", "refresh", "elevenlabs-cli"])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run snap: {}", e))?;

    if status.success() {
        print_success("Successfully updated via Snap!");
        Ok(())
    } else {
        bail!("Failed to update via Snap. Try running: sudo snap refresh elevenlabs-cli");
    }
}

fn update_via_aur() -> Result<()> {
    println!("{} Detected AUR installation (Arch Linux)", "ℹ".blue());
    println!("  Please use your AUR helper to update, e.g.:",);
    println!("    yay -S elevenlabs-cli");
    println!("    paru -S elevenlabs-cli");
    println!("    or: pacman -Syu");

    Ok(())
}

async fn update_via_binary(exe_path: &PathBuf, version: &str) -> Result<()> {
    println!("{} Detected standalone binary installation", "ℹ".blue());
    println!("  Downloading latest version from GitHub...");

    // Detect OS and architecture
    let (os, arch) = detect_os_and_arch()?;

    // Download the binary - naming convention: elevenlabs-cli-v{version}-{arch}-{os}.tar.gz
    // For Windows, use .zip instead of .tar.gz
    let extension = if cfg!(windows) { "zip" } else { "tar.gz" };
    let binary_name = format!("elevenlabs-cli-v{}-{}-{}.{}", version, arch, os, extension);
    let download_url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        GITHUB_REPO, version, binary_name
    );

    println!("  Downloading: {}", download_url.dimmed());

    let client = reqwest::Client::builder()
        .user_agent("elevenlabs-cli")
        .build()?;

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        bail!(
            "Failed to download update. Please download manually from:\n  https://github.com/{}/releases/latest",
            GITHUB_REPO
        );
    }

    download_and_replace(response, exe_path).await?;

    print_success(&format!("Successfully updated to version {}!", version));
    Ok(())
}

async fn download_and_replace(response: reqwest::Response, exe_path: &PathBuf) -> Result<()> {
    use tempfile::tempdir;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

    let temp_dir = tempdir()?;

    // Handle zip files (Windows) vs tar.gz files (Unix)
    #[cfg(windows)]
    {
        let zip_path = temp_dir.path().join("download.zip");
        std::fs::write(&zip_path, &bytes)?;

        let file = std::fs::File::open(&zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let extract_dir = temp_dir.path().join("extracted");
        std::fs::create_dir(&extract_dir)?;
        archive.extract(&extract_dir)?;

        let new_binary = find_binary_in_dir(&extract_dir)?;
        std::fs::copy(&new_binary, exe_path)?;
    }

    #[cfg(not(windows))]
    {
        use flate2::read::GzDecoder;

        let tar_path = temp_dir.path().join("download.tar.gz");
        std::fs::write(&tar_path, &bytes)?;

        // Extract the tarball
        let file = std::fs::File::open(&tar_path)?;
        let gz = GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);

        let extract_dir = temp_dir.path().join("extracted");
        std::fs::create_dir(&extract_dir)?;
        archive.unpack(&extract_dir)?;

        // Find the binary in the extracted files
        let new_binary = find_binary_in_dir(&extract_dir)?;

        // Make it executable
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&new_binary, std::fs::Permissions::from_mode(0o755))?;

        // Replace the old binary
        let backup_path = exe_path.with_extension("bak");
        std::fs::rename(exe_path, &backup_path)?;
        std::fs::copy(&new_binary, exe_path)?;
        std::fs::remove_file(backup_path)?;
    }

    Ok(())
}

fn find_binary_in_dir(dir: &std::path::Path) -> Result<PathBuf> {
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name() {
                if name == "elevenlabs-cli" || name == "elevenlabs-cli.exe" {
                    return Ok(path.to_path_buf());
                }
            }
        }
    }
    bail!("Could not find binary in downloaded archive")
}

fn detect_os_and_arch() -> Result<(&'static str, &'static str)> {
    let os = match std::env::consts::OS {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        "windows" => "pc-windows-msvc",
        _ => bail!("Unsupported OS: {}", std::env::consts::OS),
    };

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        "arm" => "arm",
        _ => bail!("Unsupported architecture: {}", std::env::consts::ARCH),
    };

    Ok((os, arch))
}
