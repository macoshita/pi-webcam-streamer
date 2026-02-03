use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result};

const SERVICE_NAME: &str = "pi-webcam-streamer";
const SERVICE_FILE_PATH: &str = "/etc/systemd/system/pi-webcam-streamer.service";

pub fn install_service() -> Result<()> {
    println!("Installing systemd service for {}...", SERVICE_NAME);

    // 1. Check root
    if !is_root() {
        anyhow::bail!("This command must be run with sudo.");
    }

    // 2. Get current executable path
    let current_exe = env::current_exe().context("Failed to get current executable path")?;
    let exe_path_str = current_exe.to_str().context("Path is not valid UTF-8")?;

    // 3. Create service file content
    let service_content = format!(
        r#"[Unit]
Description=Pi Webcam Streamer
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory={}
ExecStart={}
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#,
        current_exe.parent().unwrap_or(Path::new("/")).to_str().unwrap_or("/"),
        exe_path_str
    );

    // 4. Write service file
    fs::write(SERVICE_FILE_PATH, service_content)
        .with_context(|| format!("Failed to write service file to {}", SERVICE_FILE_PATH))?;
    println!("Created service file at {}", SERVICE_FILE_PATH);

    // 5. Reload daemon
    run_systemctl(&["daemon-reload"])?;

    // 6. Enable service
    run_systemctl(&["enable", SERVICE_NAME])?;

    // 7. Start service
    run_systemctl(&["start", SERVICE_NAME])?;

    println!("Service installed successfully!");
    Ok(())
}

pub fn uninstall_service() -> Result<()> {
    println!("Uninstalling systemd service for {}...", SERVICE_NAME);

    if !is_root() {
        anyhow::bail!("This command must be run with sudo.");
    }

    // 1. Stop service
    // Ignore error if not running
    let _ = run_systemctl(&["stop", SERVICE_NAME]);

    // 2. Disable service
    // Ignore error if not enabled
    let _ = run_systemctl(&["disable", SERVICE_NAME]);

    // 3. Remove service file
    if Path::new(SERVICE_FILE_PATH).exists() {
        fs::remove_file(SERVICE_FILE_PATH)
            .with_context(|| format!("Failed to remove service file at {}", SERVICE_FILE_PATH))?;
        println!("Removed service file at {}", SERVICE_FILE_PATH);
    } else {
        println!("Service file not found, skipping removal.");
    }

    // 4. Reload daemon
    run_systemctl(&["daemon-reload"])?;

    println!("Service uninstalled successfully!");
    Ok(())
}

fn is_root() -> bool {
    // A simple check on unix-like systems
    unsafe { libc::getuid() == 0 }
}

fn run_systemctl(args: &[&str]) -> Result<()> {
    let status = Command::new("systemctl")
        .args(args)
        .status()
        .context(format!("Failed to execute systemctl {}", args.join(" ")))?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("systemctl {} failed with {}", args.join(" "), status);
    }
}
