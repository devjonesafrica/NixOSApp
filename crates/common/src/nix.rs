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

/// Configuration options for Nix generation
#[derive(Debug, Clone, Default)]
pub struct NixGenOptions<'a> {
    pub profile: Option<&'a ProfileDef>,
    pub bundles: Vec<&'a BundleDef>,
    pub hostname: Option<&'a str>,
    pub dns_servers: Vec<String>,
    pub user_groups: Vec<String>,
    pub username: Option<&'a str>,
}

/// Generate the selected.nix file content with full options
pub fn generate_selected_nix_full(options: &NixGenOptions) -> String {
    let mut imports = Vec::new();

    // Add profile import
    if let Some(p) = options.profile {
        imports.push(format!("    ../profiles/{}.nix", p.id));
    }

    // Add bundle imports
    for bundle in &options.bundles {
        imports.push(format!("    ../bundles/{}.nix", bundle.id));
    }

    // Add hostname config if set
    if options.hostname.is_some() {
        imports.push("    ./hostname.nix".to_string());
    }

    // Add DNS config if servers are set
    if !options.dns_servers.is_empty() {
        imports.push("    ./dns.nix".to_string());
    }

    // Add user groups config if set
    if !options.user_groups.is_empty() && options.username.is_some() {
        imports.push("    ./users.nix".to_string());
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
# DNS servers: {}
# User groups: {}

{{ config, lib, pkgs, ... }}:

{{
  imports = [
{}
  ];
}}
"#,
        options.profile.map(|p| p.name.as_str()).unwrap_or("None"),
        if options.bundles.is_empty() {
            "None".to_string()
        } else {
            options.bundles
                .iter()
                .map(|b| b.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        },
        if options.dns_servers.is_empty() {
            "None".to_string()
        } else {
            options.dns_servers.join(", ")
        },
        if options.user_groups.is_empty() {
            "None".to_string()
        } else {
            options.user_groups.join(", ")
        },
        imports_str,
    )
}

/// Generate the selected.nix file content (legacy signature for compatibility)
pub fn generate_selected_nix(
    profile: Option<&ProfileDef>,
    bundles: &[&BundleDef],
    hostname: Option<&str>,
) -> String {
    let options = NixGenOptions {
        profile,
        bundles: bundles.to_vec(),
        hostname,
        ..Default::default()
    };
    generate_selected_nix_full(&options)
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

/// Generate the dns.nix file content for custom DNS configuration
pub fn generate_dns_nix(servers: &[String]) -> String {
    if servers.is_empty() {
        return String::new();
    }

    let servers_str = servers
        .iter()
        .map(|s| format!("    \"{}\"", s))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# NixOS Toolkit - DNS Configuration
# DO NOT EDIT MANUALLY

{{ config, lib, pkgs, ... }}:

{{
  networking.nameservers = [
{}
  ];
}}
"#,
        servers_str
    )
}

/// Generate the users.nix file content for user group membership
pub fn generate_user_groups_nix(username: &str, groups: &[String]) -> String {
    if username.is_empty() || groups.is_empty() {
        return String::new();
    }

    let groups_str = groups
        .iter()
        .map(|g| format!("\"{}\"", g))
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        r#"# NixOS Toolkit - User Groups Configuration
# DO NOT EDIT MANUALLY

{{ config, lib, pkgs, ... }}:

{{
  users.users.{} = {{
    extraGroups = [ {} ];
  }};
}}
"#,
        username, groups_str
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

/// Generate a preview of what will be written with full options
pub fn generate_preview_full(options: &NixGenOptions) -> String {
    let mut preview = String::new();

    preview.push_str("=== Files to be written ===\n\n");

    // selected.nix
    preview.push_str(&format!("--- {} ---\n", paths::SELECTED_NIX));
    preview.push_str(&generate_selected_nix_full(options));
    preview.push('\n');

    // hostname.nix if needed
    if let Some(h) = options.hostname {
        preview.push_str(&format!("\n--- {} ---\n", paths::HOSTNAME_NIX));
        preview.push_str(&generate_hostname_nix(h));
    }

    // dns.nix if needed
    if !options.dns_servers.is_empty() {
        preview.push_str(&format!("\n--- {} ---\n", paths::DNS_NIX));
        preview.push_str(&generate_dns_nix(&options.dns_servers));
    }

    // users.nix if needed
    if !options.user_groups.is_empty() {
        if let Some(username) = options.username {
            preview.push_str(&format!("\n--- {} ---\n", paths::USERS_NIX));
            preview.push_str(&generate_user_groups_nix(username, &options.user_groups));
        }
    }

    // Profile template (if selected and exists)
    if let Some(p) = options.profile {
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
    for bundle in &options.bundles {
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

/// Generate a preview of what will be written (legacy signature for compatibility)
pub fn generate_preview(
    profile: Option<&ProfileDef>,
    bundles: &[&BundleDef],
    hostname: Option<&str>,
) -> String {
    let options = NixGenOptions {
        profile,
        bundles: bundles.to_vec(),
        hostname,
        ..Default::default()
    };
    generate_preview_full(&options)
}
