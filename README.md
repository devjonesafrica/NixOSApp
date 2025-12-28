# NixOS Toolkit

A GTK4/libadwaita GUI application for declarative NixOS system management.

## Features

### Desktop Environment Profiles

Choose from 7 desktop environments:

| Profile | Display Manager | Session Type | Notes |
|---------|-----------------|--------------|-------|
| GNOME | GDM | Wayland | Modern, elegant desktop with GNOME Tweaks, Extension Manager |
| KDE Plasma 6 | SDDM | Wayland | Feature-rich, highly customizable with KDE Connect |
| COSMIC | cosmic-greeter | Wayland | System76's Rust-based desktop (requires NixOS with COSMIC support) |
| XFCE | LightDM | X11 | Lightweight and fast with Whisker menu, plugins |
| MATE | LightDM | X11 | Traditional GNOME 2 experience |
| Cinnamon | LightDM | X11 | Modern traditional desktop from Linux Mint |
| Pantheon | LightDM | X11 | Desktop from elementary OS with elementary apps |

### Software Bundles

Enable curated software collections. Each bundle includes more than listed here - see full details below.

| Bundle | Key Components | Also Enables |
|--------|---------------|--------------|
| Development Tools | Git, Neovim, VS Code, Rust, Node.js, Python, Go | Docker service, GitHub CLI, ripgrep, fd |
| Gaming | Steam, Lutris, MangoHud, Gamemode | Heroic launcher, Wine, Proton tools, 32-bit graphics |
| Virtualization (KVM) | virt-manager, QEMU, libvirt | OVMF (UEFI), SPICE, TPM emulation, Windows VirtIO drivers |
| VirtualBox | Oracle VirtualBox | Extension Pack (non-free), guest additions, sets `allowUnfree` |
| Containers | Podman, docker-compose, buildah, skopeo | Uses Podman with Docker compatibility alias |
| Flatpak | Flatpak, gnome-software | XDG portals |
| Multimedia | VLC, mpv, GIMP, Inkscape, OBS Studio | PipeWire audio, Krita, GStreamer codecs |
| Office | LibreOffice, Thunderbird, Obsidian | OnlyOffice, printing/scanning support, PDF tools |

### System Configuration

- **Hostname**: Change the system hostname
- **Custom DNS**: Set a custom DNS resolver
- **User Groups**: Add users to libvirtd, docker, or vboxusers groups

### Maintenance Tools

- Garbage collection (unreachable objects)
- Delete old generations
- Optimize Nix store (deduplication)
- Verify store integrity
- Update channels

### Core Design

- **Safe Integration**: All changes are managed in `/etc/nixos/nixos-toolkit/`, never overwriting your existing config
- **Live Preview**: See exactly what Nix code will be generated before applying
- **Declarative**: All changes integrate with NixOS's generation system for easy rollback
- **Privilege Separation**: GUI runs unprivileged; privileged operations use pkexec

## Quick Start

### Run without installing

```bash
nix run github:devjonesafrica/NixOSApp
```

### Install to your profile

```bash
nix profile install github:devjonesafrica/NixOSApp
```

### Add to your NixOS configuration

```nix
# In your flake.nix inputs:
inputs.nixos-toolkit.url = "github:devjonesafrica/NixOSApp";

# In your nixosConfigurations:
modules = [
  nixos-toolkit.nixosModules.default
  # ...
];

# In your configuration.nix or system config:
programs.nixos-toolkit.enable = true;
```

## One-Time Setup

The toolkit manages configuration in `/etc/nixos/nixos-toolkit/`. You need to add a single import to your existing configuration:

### Classic Configuration (configuration.nix)

Add this import to your `/etc/nixos/configuration.nix`:

```nix
{ config, pkgs, ... }:

{
  imports = [
    ./hardware-configuration.nix
    ./nixos-toolkit/state/selected.nix  # Add this line
  ];

  # ... rest of your configuration
}
```

### Flake-Based Configuration

Add this to your `flake.nix` nixosConfigurations:

```nix
nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
  modules = [
    ./configuration.nix
    ./nixos-toolkit/state/selected.nix  # Add this line
  ];
};
```

After adding the import, run `sudo nixos-rebuild switch` once. The toolkit will detect the integration and enable the Apply button.

## Development

### Prerequisites

- NixOS with flakes enabled
- Nix 2.4+

### Enter development shell

```bash
nix develop
```

### Build and run

```bash
# Run the GUI
cargo run -p gui

# Run the helper (requires root for actual operations)
cargo run -p helper

# Build all packages
nix build

# Build specific package
nix build .#gui
nix build .#helper
```

### Project Structure

```
.
├── Cargo.toml              # Workspace configuration (Rust 1.75+, GPL-3.0)
├── flake.nix               # Nix flake with build definitions
├── crates/
│   ├── common/             # Shared types and utilities
│   │   └── src/
│   │       ├── actions.rs  # Action registry (7 profiles, 8 bundles, 5 system actions, 5 maintenance actions)
│   │       ├── ipc.rs      # IPC message types (JSON over stdin/stdout)
│   │       ├── config.rs   # System configuration types and paths
│   │       └── nix.rs      # Nix code generation
│   ├── gui/                # GTK4/libadwaita GUI application
│   │   └── src/
│   │       ├── app.rs      # Application setup
│   │       ├── window.rs   # Main window with sidebar navigation
│   │       ├── pages/      # UI pages (Onboarding, Profiles, Bundles, System, Maintenance, Apply)
│   │       └── helper/     # Helper process communication via pkexec
│   └── helper/             # Privileged helper binary
│       └── src/
│           ├── commands.rs # Command handlers
│           ├── nix_gen.rs  # Nix file generation with fallback templates
│           └── rebuild.rs  # nixos-rebuild execution
├── nix/
│   └── templates/          # Nix module templates
│       ├── profiles/       # 7 desktop environment profiles
│       ├── bundles/        # 8 software bundles
│       └── state/          # State file templates
└── data/
    ├── nixos-toolkit.desktop  # Desktop entry
    ├── icons/                 # Application icon (SVG)
    └── polkit/                # Polkit policy for privilege escalation
```

## Architecture

```
┌─────────────────────┐     JSON/stdin-stdout   ┌─────────────────────┐
│   nixos-toolkit     │ ───────────────────────►│ nixos-toolkit-helper│
│   (GUI, unprivileged)│       via pkexec        │   (privileged)      │
└─────────────────────┘                         └─────────────────────┘
         │                                                │
         │ reads                                          │ writes
         ▼                                                ▼
┌─────────────────────┐                         ┌─────────────────────┐
│ /etc/nixos/         │                         │/etc/nixos/nixos-    │
│ configuration.nix   │ ◄── imports ─────────── │toolkit/             │
│ (user-managed)      │                         │  state/selected.nix │
└─────────────────────┘                         │  profiles/*.nix     │
                                                │  bundles/*.nix      │
                                                └─────────────────────┘
```

## Adding New Profiles/Bundles

### Add a new profile

1. Create a template file in `nix/templates/profiles/your-profile.nix`
2. Add the profile definition in `crates/common/src/actions.rs` to `default_profiles()`:

```rust
ProfileDef {
    id: "your-profile".into(),
    name: "Your Profile".into(),
    description: "Description of the profile".into(),
    icon: "desktop-symbolic".into(),
    template: "profiles/your-profile.nix".into(),
    display_manager: "gdm".into(),
}
```

### Add a new bundle

1. Create a template file in `nix/templates/bundles/your-bundle.nix`
2. Add the bundle definition in `crates/common/src/actions.rs` to `default_bundles()`:

```rust
BundleDef {
    id: "your-bundle".into(),
    name: "Your Bundle".into(),
    description: "Description of the bundle".into(),
    icon: "icon-name-symbolic".into(),
    category: ActionCategory::YourCategory,
    template: "bundles/your-bundle.nix".into(),
    packages: vec!["pkg1".into(), "pkg2".into()],
}
```

## Security Model

- The GUI runs as an unprivileged user
- Privileged operations (file writes, nixos-rebuild) are handled by a separate helper binary
- The helper is invoked via `pkexec` for authentication (polkit)
- The toolkit NEVER modifies your existing configuration.nix or flake.nix
- All managed files are in `/etc/nixos/nixos-toolkit/` and marked as auto-generated
- Changes integrate with NixOS generations for rollback capability

## Roadmap

### Completed

- [x] Onboarding with integration detection
- [x] 7 Desktop profiles (GNOME, KDE Plasma 6, COSMIC, XFCE, MATE, Cinnamon, Pantheon)
- [x] 8 Software bundles (DevTools, Gaming, KVM, VirtualBox, Containers, Flatpak, Multimedia, Office)
- [x] Wayland-first configuration for GNOME, KDE, COSMIC
- [x] Hostname configuration
- [x] Configuration preview
- [x] Helper binary for privileged operations via pkexec

### Partially Implemented

- [ ] Custom DNS configuration (UI exists, Nix generation unclear)
- [ ] User group management (UI exists, Nix generation unclear)
- [ ] Maintenance tools (UI exists, execution mechanism unclear)

### Planned

- [ ] Full polkit integration (D-Bus service instead of pkexec)
- [ ] More profiles (Hyprland, Sway, i3, Budgie, LXQt)
- [ ] Home-manager integration
- [ ] Generation history and rollback UI
- [ ] Theme/appearance settings
- [ ] Network/firewall configuration
- [ ] Service toggles (SSH, printing, etc.)
- [ ] Package search and management

## Known Issues

1. **Channel updates on flakes**: The "Update Channels" maintenance action uses `nix-channel --update` which may not be relevant for flake-based configurations.

## License

GPL-3.0-or-later

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.
