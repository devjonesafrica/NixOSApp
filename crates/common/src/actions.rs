//! Action registry and definitions for NixOS Toolkit
//!
//! This module defines the extensible action system that allows adding
//! new profiles, bundles, and system actions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for an action
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionId(pub String);

impl ActionId {
    pub fn new(category: &str, name: &str) -> Self {
        Self(format!("{}.{}", category, name))
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl std::fmt::Display for ActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Categories for organizing actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionCategory {
    Desktop,
    Development,
    Gaming,
    Multimedia,
    Virtualization,
    System,
    Network,
    Security,
    Office,
}

impl ActionCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Desktop => "Desktop Environments",
            Self::Development => "Development Tools",
            Self::Gaming => "Gaming",
            Self::Multimedia => "Multimedia",
            Self::Virtualization => "Virtualization",
            Self::System => "System Settings",
            Self::Network => "Networking",
            Self::Security => "Security",
            Self::Office => "Office & Productivity",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Desktop => "user-desktop-symbolic",
            Self::Development => "applications-engineering-symbolic",
            Self::Gaming => "applications-games-symbolic",
            Self::Multimedia => "applications-multimedia-symbolic",
            Self::Virtualization => "computer-symbolic",
            Self::System => "preferences-system-symbolic",
            Self::Network => "network-wired-symbolic",
            Self::Security => "security-high-symbolic",
            Self::Office => "x-office-document-symbolic",
        }
    }
}

/// Configuration value types for action settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Bool(bool),
    String(String),
    Integer(i64),
    Float(f64),
    List(Vec<ConfigValue>),
}

/// User-provided configuration for an action
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionConfig {
    pub enabled: bool,
    pub values: HashMap<String, ConfigValue>,
}

impl ActionConfig {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            values: HashMap::new(),
        }
    }

    pub fn with_value(mut self, key: impl Into<String>, value: ConfigValue) -> Self {
        self.values.insert(key.into(), value);
        self
    }
}

/// Metadata describing an action for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMetadata {
    pub id: ActionId,
    pub name: String,
    pub description: String,
    pub category: ActionCategory,
    pub icon: Option<String>,
    pub template_path: Option<String>,
    pub conflicts: Vec<ActionId>,
    pub requires: Vec<ActionId>,
}

/// Definition of a desktop environment profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub template: String,
    pub display_manager: String,
}

/// Definition of a software bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: ActionCategory,
    pub template: String,
    pub packages: Vec<String>,
}

/// Get all available profile definitions
pub fn default_profiles() -> Vec<ProfileDef> {
    vec![
        ProfileDef {
            id: "gnome".into(),
            name: "GNOME".into(),
            description: "Modern, elegant desktop environment with a focus on simplicity and productivity".into(),
            icon: "desktop-symbolic".into(),
            template: "profiles/gnome.nix".into(),
            display_manager: "gdm".into(),
        },
        ProfileDef {
            id: "kde".into(),
            name: "KDE Plasma".into(),
            description: "Feature-rich, highly customizable desktop environment".into(),
            icon: "desktop-symbolic".into(),
            template: "profiles/kde.nix".into(),
            display_manager: "sddm".into(),
        },
        ProfileDef {
            id: "xfce".into(),
            name: "XFCE".into(),
            description: "Lightweight, fast desktop environment with traditional desktop experience".into(),
            icon: "desktop-symbolic".into(),
            template: "profiles/xfce.nix".into(),
            display_manager: "lightdm".into(),
        },
        ProfileDef {
            id: "mate".into(),
            name: "MATE".into(),
            description: "Traditional desktop based on GNOME 2, intuitive and lightweight".into(),
            icon: "desktop-symbolic".into(),
            template: "profiles/mate.nix".into(),
            display_manager: "lightdm".into(),
        },
        ProfileDef {
            id: "cinnamon".into(),
            name: "Cinnamon".into(),
            description: "Modern desktop from Linux Mint with traditional layout".into(),
            icon: "desktop-symbolic".into(),
            template: "profiles/cinnamon.nix".into(),
            display_manager: "lightdm".into(),
        },
        ProfileDef {
            id: "pantheon".into(),
            name: "Pantheon".into(),
            description: "Beautiful, clean desktop from elementary OS".into(),
            icon: "desktop-symbolic".into(),
            template: "profiles/pantheon.nix".into(),
            display_manager: "lightdm".into(),
        },
        ProfileDef {
            id: "cosmic".into(),
            name: "COSMIC".into(),
            description: "System76's modern, Rust-based Wayland desktop (stable 1.0)".into(),
            icon: "desktop-symbolic".into(),
            template: "profiles/cosmic.nix".into(),
            display_manager: "cosmic-greeter".into(),
        },
    ]
}

/// Get all available bundle definitions
pub fn default_bundles() -> Vec<BundleDef> {
    vec![
        // Development
        BundleDef {
            id: "devtools".into(),
            name: "Development Tools".into(),
            description: "Essential development tools: Git, editors, compilers, and containers".into(),
            icon: "applications-engineering-symbolic".into(),
            category: ActionCategory::Development,
            template: "bundles/devtools.nix".into(),
            packages: vec![
                "git".into(),
                "neovim".into(),
                "vscode".into(),
                "rustup".into(),
                "nodejs".into(),
                "python3".into(),
                "docker".into(),
            ],
        },
        // Gaming
        BundleDef {
            id: "gaming".into(),
            name: "Gaming".into(),
            description: "Steam, Lutris, and gaming utilities for Linux gaming".into(),
            icon: "applications-games-symbolic".into(),
            category: ActionCategory::Gaming,
            template: "bundles/gaming.nix".into(),
            packages: vec![
                "steam".into(),
                "lutris".into(),
                "mangohud".into(),
                "gamemode".into(),
            ],
        },
        // Virtualization
        BundleDef {
            id: "virtualization".into(),
            name: "KVM/QEMU Virtualization".into(),
            description: "Hardware-accelerated VMs with libvirt and virt-manager".into(),
            icon: "computer-symbolic".into(),
            category: ActionCategory::Virtualization,
            template: "bundles/virtualization.nix".into(),
            packages: vec![
                "virt-manager".into(),
                "qemu".into(),
                "OVMF".into(),
                "spice-gtk".into(),
            ],
        },
        BundleDef {
            id: "virtualbox".into(),
            name: "VirtualBox".into(),
            description: "Oracle VirtualBox with Extension Pack (non-free)".into(),
            icon: "computer-symbolic".into(),
            category: ActionCategory::Virtualization,
            template: "bundles/virtualbox.nix".into(),
            packages: vec![
                "virtualbox".into(),
            ],
        },
        BundleDef {
            id: "containers".into(),
            name: "Container Runtime".into(),
            description: "Podman/Docker container support with compose".into(),
            icon: "package-x-generic-symbolic".into(),
            category: ActionCategory::Virtualization,
            template: "bundles/containers.nix".into(),
            packages: vec![
                "podman".into(),
                "docker-compose".into(),
                "buildah".into(),
                "skopeo".into(),
            ],
        },
        // System
        BundleDef {
            id: "flatpak".into(),
            name: "Flatpak Support".into(),
            description: "Enable Flatpak for sandboxed application installation".into(),
            icon: "package-x-generic-symbolic".into(),
            category: ActionCategory::System,
            template: "bundles/flatpak.nix".into(),
            packages: vec![
                "flatpak".into(),
            ],
        },
        // Multimedia
        BundleDef {
            id: "multimedia".into(),
            name: "Multimedia".into(),
            description: "Audio, video, and graphics applications with PipeWire".into(),
            icon: "applications-multimedia-symbolic".into(),
            category: ActionCategory::Multimedia,
            template: "bundles/multimedia.nix".into(),
            packages: vec![
                "vlc".into(),
                "mpv".into(),
                "gimp".into(),
                "inkscape".into(),
                "obs-studio".into(),
                "audacity".into(),
            ],
        },
        // Office
        BundleDef {
            id: "office".into(),
            name: "Office & Productivity".into(),
            description: "LibreOffice, PDF tools, email, and productivity apps".into(),
            icon: "x-office-document-symbolic".into(),
            category: ActionCategory::Office,
            template: "bundles/office.nix".into(),
            packages: vec![
                "libreoffice".into(),
                "thunderbird".into(),
                "evince".into(),
                "obsidian".into(),
            ],
        },
    ]
}

/// System action definition (hostname, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemActionDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub action_type: SystemActionType,
}

/// Type of system action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemActionType {
    /// Text input (hostname, DNS IP)
    TextInput { placeholder: String },
    /// Toggle on/off
    Toggle,
    /// Selection from options
    Select { options: Vec<String> },
    /// User selection for group membership
    UserGroup { group: String },
}

/// Get available system actions
pub fn default_system_actions() -> Vec<SystemActionDef> {
    vec![
        SystemActionDef {
            id: "hostname".into(),
            name: "Hostname".into(),
            description: "Change the system hostname".into(),
            icon: "computer-symbolic".into(),
            action_type: SystemActionType::TextInput {
                placeholder: "nixos".into(),
            },
        },
        SystemActionDef {
            id: "dns".into(),
            name: "Custom DNS".into(),
            description: "Set a custom DNS resolver (e.g., 1.1.1.1, 8.8.8.8)".into(),
            icon: "network-server-symbolic".into(),
            action_type: SystemActionType::TextInput {
                placeholder: "1.1.1.1".into(),
            },
        },
        SystemActionDef {
            id: "libvirtd_user".into(),
            name: "libvirtd Group".into(),
            description: "Add user to libvirtd group for KVM access".into(),
            icon: "system-users-symbolic".into(),
            action_type: SystemActionType::UserGroup {
                group: "libvirtd".into(),
            },
        },
        SystemActionDef {
            id: "docker_user".into(),
            name: "Docker Group".into(),
            description: "Add user to docker group for rootless Docker".into(),
            icon: "system-users-symbolic".into(),
            action_type: SystemActionType::UserGroup {
                group: "docker".into(),
            },
        },
        SystemActionDef {
            id: "vboxusers".into(),
            name: "VirtualBox Group".into(),
            description: "Add user to vboxusers group for VirtualBox access".into(),
            icon: "system-users-symbolic".into(),
            action_type: SystemActionType::UserGroup {
                group: "vboxusers".into(),
            },
        },
    ]
}

/// Maintenance action definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceActionDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub warning: Option<String>,
    pub command: String,
}

/// Get available maintenance actions
pub fn default_maintenance_actions() -> Vec<MaintenanceActionDef> {
    vec![
        MaintenanceActionDef {
            id: "gc_unreachable".into(),
            name: "Garbage Collect".into(),
            description: "Delete unreachable store objects to free disk space".into(),
            icon: "user-trash-symbolic".into(),
            warning: None,
            command: "nix-collect-garbage".into(),
        },
        MaintenanceActionDef {
            id: "gc_all".into(),
            name: "Delete Old Generations".into(),
            description: "Delete all old system generations (keeps current only)".into(),
            icon: "user-trash-full-symbolic".into(),
            warning: Some("This will delete all old configurations. You won't be able to roll back to previous generations!".into()),
            command: "nix-collect-garbage -d".into(),
        },
        MaintenanceActionDef {
            id: "optimize_store".into(),
            name: "Optimize Store".into(),
            description: "Deduplicate files in the Nix store to save space".into(),
            icon: "drive-harddisk-symbolic".into(),
            warning: None,
            command: "nix-store --optimise".into(),
        },
        MaintenanceActionDef {
            id: "verify_store".into(),
            name: "Verify Store".into(),
            description: "Check Nix store for integrity issues".into(),
            icon: "security-high-symbolic".into(),
            warning: None,
            command: "nix-store --verify --check-contents".into(),
        },
        MaintenanceActionDef {
            id: "update_channels".into(),
            name: "Update Channels".into(),
            description: "Update Nix channels to latest versions".into(),
            icon: "software-update-available-symbolic".into(),
            warning: None,
            command: "nix-channel --update".into(),
        },
    ]
}
