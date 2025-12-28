# GNOME Desktop Profile
# Managed by NixOS Toolkit
#
# This profile enables the GNOME desktop environment with common utilities.
# It provides a modern, polished desktop experience with Wayland by default.

{ config, lib, pkgs, ... }:

{
  # Enable GDM display manager
  services.xserver.enable = true;
  services.xserver.displayManager.gdm.enable = true;
  services.xserver.displayManager.gdm.wayland = true;

  # Enable GNOME desktop environment
  services.xserver.desktopManager.gnome.enable = true;

  # GNOME core utilities
  services.gnome.core-utilities.enable = true;

  # Additional GNOME packages
  environment.systemPackages = with pkgs; [
    # GNOME utilities
    gnome-tweaks
    dconf-editor
    gnome-extension-manager

    # Common applications
    gnome-calculator
    gnome-system-monitor
    gnome-disk-utility
    file-roller
    loupe  # Modern image viewer
  ];

  # Enable dconf for GNOME settings
  programs.dconf.enable = true;

  # XDG portal for Flatpak/sandboxed apps
  xdg.portal = {
    enable = true;
    extraPortals = [ pkgs.xdg-desktop-portal-gnome ];
  };
}
