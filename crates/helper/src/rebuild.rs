//! nixos-rebuild execution

use common::config::ConfigMode;
use common::ipc::{HelperResponse, LogLevel, RebuildType};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

/// Run nixos-rebuild with the specified type
pub fn run_rebuild(rebuild_type: RebuildType, config_mode: ConfigMode) -> HelperResponse {
    let rebuild_arg = rebuild_type.as_arg();

    // Build command based on config mode
    let mut cmd = Command::new("nixos-rebuild");
    cmd.arg(rebuild_arg);

    match config_mode {
        ConfigMode::Flake => {
            // Get hostname for flake reference
            let hostname = std::fs::read_to_string("/etc/hostname")
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "nixos".to_string());

            cmd.arg("--flake")
                .arg(format!("/etc/nixos#{}", hostname));
        }
        ConfigMode::Classic | ConfigMode::Unknown => {
            // Classic mode uses default paths
        }
    }

    // Capture output
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    tracing::info!("Running: nixos-rebuild {}", rebuild_arg);

    // Spawn the process
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return HelperResponse::Error {
                message: "Failed to spawn nixos-rebuild".into(),
                details: Some(e.to_string()),
            };
        }
    };

    // Stream output
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let mut stdout_handle = std::io::stdout();

    // Stream stdout
    if let Some(stdout) = stdout {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            let log = HelperResponse::Log {
                level: LogLevel::Info,
                message: line,
            };
            if let Ok(json) = serde_json::to_string(&log) {
                let _ = writeln!(stdout_handle, "{}", json);
                let _ = stdout_handle.flush();
            }
        }
    }

    // Stream stderr
    if let Some(stderr) = stderr {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            let level = if line.contains("error") || line.contains("Error") {
                LogLevel::Error
            } else if line.contains("warning") || line.contains("Warning") {
                LogLevel::Warning
            } else {
                LogLevel::Info
            };

            let log = HelperResponse::Log {
                level,
                message: line,
            };
            if let Ok(json) = serde_json::to_string(&log) {
                let _ = writeln!(stdout_handle, "{}", json);
                let _ = stdout_handle.flush();
            }
        }
    }

    // Wait for completion
    match child.wait() {
        Ok(status) => {
            if status.success() {
                tracing::info!("nixos-rebuild completed successfully");
                HelperResponse::ApplyComplete {
                    success: true,
                    message: format!("nixos-rebuild {} completed successfully", rebuild_arg),
                }
            } else {
                let code = status.code().unwrap_or(-1);
                tracing::error!("nixos-rebuild failed with code {}", code);
                HelperResponse::ApplyComplete {
                    success: false,
                    message: format!("nixos-rebuild {} failed with exit code {}", rebuild_arg, code),
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to wait for nixos-rebuild: {}", e);
            HelperResponse::Error {
                message: "Failed to wait for nixos-rebuild".into(),
                details: Some(e.to_string()),
            }
        }
    }
}
