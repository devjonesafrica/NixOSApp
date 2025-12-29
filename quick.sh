#!/usr/bin/env bash
# NixOS Toolkit Quick Installer
set -e

REPO="github:devjonesafrica/NixOSApp"

echo "==================================="
echo "  NixOS Toolkit Installer"
echo "==================================="

# Check flakes
if nix flake --version >/dev/null 2>&1; then
    echo ""
    echo "Flakes enabled! Run with:"
    echo "  nix run $REPO"
    CMD="nix run $REPO"
else
    echo ""
    echo "Flakes not enabled. Run with:"
    echo "  nix --extra-experimental-features 'nix-command flakes' run $REPO"
    echo ""
    echo "Or enable flakes permanently in /etc/nixos/configuration.nix:"
    echo "  nix.settings.experimental-features = [ \"nix-command\" \"flakes\" ];"
    CMD="nix --extra-experimental-features 'nix-command flakes' run $REPO"
fi

echo ""
echo "-----------------------------------"
echo "Create alias 'nixos-toolkit'? [Y/n]"
read REPLY

if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    if [ -f "$HOME/.zshrc" ]; then
        RC="$HOME/.zshrc"
    else
        RC="$HOME/.bashrc"
    fi

    if ! grep -q "alias nixos-toolkit=" "$RC" 2>/dev/null; then
        echo "" >> "$RC"
        echo "# NixOS Toolkit" >> "$RC"
        echo "alias nixos-toolkit='$CMD'" >> "$RC"
        echo "Added to $RC - restart terminal or run: source $RC"
    else
        echo "Alias already exists"
    fi
fi

echo ""
echo "Done! Run: nixos-toolkit"
