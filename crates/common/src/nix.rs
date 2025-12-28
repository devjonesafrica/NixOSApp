//! Nix code generation utilities

use crate::actions::{BundleDef, ProfileDef};
use crate::config::paths;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during Nix generation
#[derive(Debug, Error)]
pub enum NixGenError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Failed to read template: {0}")]
    ReadError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Output from Nix generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NixOutput {
    /// The generated Nix code
    pub content: String,
    /// Path where this should be written
    pub path: String,
}

/// Generate the selected.nix file content
pub fn generate_selected_nix(
    profile: Option<&ProfileDef>,
    bundles: &[&BundleDef],
    hostname: Option<&str>,
) -> String {
    let mut imports = Vec::new();
    let mut inline_config = String::new();

    // Add profile import
    if let Some(p) = profile {
        imports.push(format!("    ../profiles/{}.nix", p.id));
    }

    // Add bundle imports
    for bundle in bundles {
        imports.push(format!("    ../bundles/{}.nix", bundle.id));
    }

    // Add hostname config if set
    if hostname.is_some() {
        imports.push("    ./hostname.nix".to_string());
    }

    // Build the imports section
    let imports_str = if imports.is_empty() {
        "    # No profiles or bundles selected".to_string()
    } else {
        imports.join("\n")
    };

    format!(
        r#"# NixOS Toolkit - Managed Configuration
# DO NOT EDIT MANUALLY - Changes will be overwritten
#
# This file is managed by nixos-toolkit. To make changes,
# use the GUI application and click "Apply".
#
# Selected profile: {}
# Enabled bundles: {}

{{ config, lib, pkgs, ... }}:

{{
  imports = [
{}
  ];
{}}}
"#,
        profile.map(|p| p.name.as_str()).unwrap_or("None"),
        if bundles.is_empty() {
            "None".to_string()
        } else {
            bundles
                .iter()
                .map(|b| b.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        },
        imports_str,
        if inline_config.is_empty() {
            ""
        } else {
            &inline_config
        }
    )
}

/// Generate the hostname.nix file content
pub fn generate_hostname_nix(hostname: &str) -> String {
    format!(
        r#"# NixOS Toolkit - Hostname Configuration
# DO NOT EDIT MANUALLY

{{ config, lib, pkgs, ... }}:

{{
  networking.hostName = "{}";
}}
"#,
        hostname
    )
}

/// Read a template file from the templates directory
pub fn read_template(template_path: &str) -> Result<String, NixGenError> {
    let templates_dir = paths::templates_dir();
    let full_path = templates_dir.join(template_path);

    std::fs::read_to_string(&full_path).map_err(|e| {
        NixGenError::ReadError(format!("{}: {}", full_path.display(), e))
    })
}

/// Check if a template exists
pub fn template_exists(template_path: &str) -> bool {
    let templates_dir = paths::templates_dir();
    templates_dir.join(template_path).exists()
}

/// Generate a preview of what will be written
pub fn generate_preview(
    profile: Option<&ProfileDef>,
    bundles: &[&BundleDef],
    hostname: Option<&str>,
) -> String {
    let mut preview = String::new();

    preview.push_str("=== Files to be written ===\n\n");

    // selected.nix
    preview.push_str(&format!("--- {} ---\n", paths::SELECTED_NIX));
    preview.push_str(&generate_selected_nix(profile, bundles, hostname));
    preview.push('\n');

    // hostname.nix if needed
    if let Some(h) = hostname {
        preview.push_str(&format!("\n--- {} ---\n", paths::HOSTNAME_NIX));
        preview.push_str(&generate_hostname_nix(h));
    }

    // Profile template (if selected and exists)
    if let Some(p) = profile {
        if template_exists(&p.template) {
            if let Ok(content) = read_template(&p.template) {
                preview.push_str(&format!(
                    "\n--- /etc/nixos/nixos-toolkit/profiles/{}.nix ---\n",
                    p.id
                ));
                preview.push_str(&content);
            }
        }
    }

    // Bundle templates
    for bundle in bundles {
        if template_exists(&bundle.template) {
            if let Ok(content) = read_template(&bundle.template) {
                preview.push_str(&format!(
                    "\n--- /etc/nixos/nixos-toolkit/bundles/{}.nix ---\n",
                    bundle.id
                ));
                preview.push_str(&content);
            }
        }
    }

    preview
}
