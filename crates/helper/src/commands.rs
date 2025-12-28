//! Command handlers for helper requests

use crate::nix_gen;
use crate::rebuild;
use common::config::{paths, ConfigMode, IntegrationStatus, SystemInfo};
use common::ipc::{AppState, GeneratedFile, HelperResponse, RebuildType};
use std::fs;
use std::path::Path;

/// Check if we have the required permissions
pub fn check_permissions() -> HelperResponse {
    let can_read_config = Path::new("/etc/nixos").exists()
        && fs::read_dir("/etc/nixos").is_ok();

    let can_write_managed = check_write_permission(paths::MANAGED_DIR)
        || check_write_permission("/etc/nixos");

    let can_run_rebuild = which("nixos-rebuild").is_some();

    HelperResponse::Permissions {
        can_read_config,
        can_write_managed,
        can_run_rebuild,
    }
}

fn check_write_permission(path: &str) -> bool {
    let p = Path::new(path);
    if p.exists() {
        // Try to open for writing
        fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(p.join(".write_test"))
            .map(|_| {
                let _ = fs::remove_file(p.join(".write_test"));
                true
            })
            .unwrap_or(false)
    } else {
        // Check parent
        if let Some(parent) = p.parent() {
            check_write_permission(parent.to_str().unwrap_or(""))
        } else {
            false
        }
    }
}

fn which(cmd: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(cmd);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

/// Get system information
pub fn get_system_info() -> HelperResponse {
    let is_nixos = Path::new("/etc/NIXOS").exists();

    let nixos_version = fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|l| l.starts_with("VERSION_ID="))
                .map(|l| l.trim_start_matches("VERSION_ID=").trim_matches('"').to_string())
        });

    let config_mode = if Path::new("/etc/nixos/flake.nix").exists() {
        ConfigMode::Flake
    } else if Path::new("/etc/nixos/configuration.nix").exists() {
        ConfigMode::Classic
    } else {
        ConfigMode::Unknown
    };

    let config_path = if Path::new("/etc/nixos/flake.nix").exists() {
        Some("/etc/nixos/flake.nix".into())
    } else if Path::new("/etc/nixos/configuration.nix").exists() {
        Some("/etc/nixos/configuration.nix".into())
    } else {
        None
    };

    let integration_status = detect_integration();

    let hostname = fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .ok();

    let current_desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();

    HelperResponse::SystemInfo(SystemInfo {
        is_nixos,
        nixos_version,
        config_mode,
        config_path,
        integration_status,
        hostname,
        current_desktop,
    })
}

fn detect_integration() -> IntegrationStatus {
    // Check if selected.nix exists
    if !Path::new(paths::SELECTED_NIX).exists() {
        return IntegrationStatus::NotIntegrated;
    }

    // Check if it's imported in configuration.nix
    if let Ok(content) = fs::read_to_string("/etc/nixos/configuration.nix") {
        if content.contains("nixos-toolkit") {
            return IntegrationStatus::Integrated;
        }
    }

    // Check flake.nix
    if let Ok(content) = fs::read_to_string("/etc/nixos/flake.nix") {
        if content.contains("nixos-toolkit") {
            return IntegrationStatus::Integrated;
        }
    }

    IntegrationStatus::NotIntegrated
}

/// Validate a configuration
pub fn validate(
    selected_profile: Option<String>,
    enabled_bundles: Vec<String>,
    hostname: Option<String>,
) -> HelperResponse {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Validate profile
    if let Some(ref profile) = selected_profile {
        let valid_profiles = [
            "gnome", "kde", "xfce", "mate", "cinnamon", "pantheon", "cosmic",
        ];
        if !valid_profiles.contains(&profile.as_str()) {
            errors.push(format!("Unknown profile: {}", profile));
        }
    }

    // Validate bundles
    let valid_bundles = [
        "devtools", "gaming", "virtualization", "virtualbox",
        "containers", "flatpak", "multimedia", "office",
    ];
    for bundle in &enabled_bundles {
        if !valid_bundles.contains(&bundle.as_str()) {
            warnings.push(format!("Unknown bundle: {}", bundle));
        }
    }

    // Validate hostname
    if let Some(ref h) = hostname {
        if h.is_empty() {
            errors.push("Hostname cannot be empty".into());
        } else if !h.chars().all(|c| c.is_alphanumeric() || c == '-') {
            errors.push("Hostname contains invalid characters".into());
        } else if h.len() > 63 {
            errors.push("Hostname too long (max 63 characters)".into());
        }
    }

    // Check integration
    if !Path::new(paths::MANAGED_DIR).exists() {
        warnings.push("Managed directory does not exist yet".into());
    }

    HelperResponse::ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Generate configuration files
pub fn generate(
    selected_profile: Option<String>,
    enabled_bundles: Vec<String>,
    hostname: Option<String>,
    dry_run: bool,
) -> HelperResponse {
    // First ensure directories exist
    if !dry_run {
        if let HelperResponse::Error { message, details } = ensure_directories() {
            return HelperResponse::Error { message, details };
        }
    }

    // Generate files
    match nix_gen::generate_all_files(&selected_profile, &enabled_bundles, hostname.as_deref(), dry_run)
    {
        Ok(files) => {
            // Generate preview
            let preview = files
                .iter()
                .map(|f| format!("=== {} ===\n{}\n", f.path, f.content))
                .collect::<Vec<_>>()
                .join("\n");

            HelperResponse::GenerationResult { files, preview }
        }
        Err(e) => HelperResponse::Error {
            message: "Failed to generate configuration".into(),
            details: Some(e.to_string()),
        },
    }
}

/// Apply configuration
pub fn apply(
    selected_profile: Option<String>,
    enabled_bundles: Vec<String>,
    hostname: Option<String>,
    rebuild_type: RebuildType,
) -> HelperResponse {
    // First generate files (not dry run)
    match nix_gen::generate_all_files(&selected_profile, &enabled_bundles, hostname.as_deref(), false)
    {
        Ok(_) => {}
        Err(e) => {
            return HelperResponse::Error {
                message: "Failed to generate configuration".into(),
                details: Some(e.to_string()),
            };
        }
    }

    // Detect config mode for rebuild
    let config_mode = if Path::new("/etc/nixos/flake.nix").exists() {
        ConfigMode::Flake
    } else {
        ConfigMode::Classic
    };

    // Run nixos-rebuild
    rebuild::run_rebuild(rebuild_type, config_mode)
}

/// Ensure required directories exist
pub fn ensure_directories() -> HelperResponse {
    let dirs = [
        paths::MANAGED_DIR,
        paths::STATE_DIR,
        paths::PROFILES_DIR,
        paths::BUNDLES_DIR,
    ];

    for dir in dirs {
        if let Err(e) = fs::create_dir_all(dir) {
            return HelperResponse::Error {
                message: format!("Failed to create directory: {}", dir),
                details: Some(e.to_string()),
            };
        }
    }

    HelperResponse::Ok
}

/// Read application state from state.json
pub fn read_state() -> HelperResponse {
    match fs::read_to_string(paths::STATE_JSON) {
        Ok(content) => match serde_json::from_str::<AppState>(&content) {
            Ok(state) => HelperResponse::State(state),
            Err(e) => HelperResponse::Error {
                message: "Failed to parse state".into(),
                details: Some(e.to_string()),
            },
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            HelperResponse::State(AppState::default())
        }
        Err(e) => HelperResponse::Error {
            message: "Failed to read state".into(),
            details: Some(e.to_string()),
        },
    }
}

/// Write application state to state.json
pub fn write_state(state: AppState) -> HelperResponse {
    // Ensure directory exists
    if let Err(e) = fs::create_dir_all(paths::STATE_DIR) {
        return HelperResponse::Error {
            message: "Failed to create state directory".into(),
            details: Some(e.to_string()),
        };
    }

    match serde_json::to_string_pretty(&state) {
        Ok(content) => {
            // Atomic write
            let temp_path = format!("{}.tmp", paths::STATE_JSON);
            if let Err(e) = fs::write(&temp_path, &content) {
                return HelperResponse::Error {
                    message: "Failed to write state".into(),
                    details: Some(e.to_string()),
                };
            }
            if let Err(e) = fs::rename(&temp_path, paths::STATE_JSON) {
                return HelperResponse::Error {
                    message: "Failed to save state".into(),
                    details: Some(e.to_string()),
                };
            }
            HelperResponse::Ok
        }
        Err(e) => HelperResponse::Error {
            message: "Failed to serialize state".into(),
            details: Some(e.to_string()),
        },
    }
}
