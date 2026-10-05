# SPDX-License-Identifier: GPL-3.0-or-later
# Omarchy-style Hyprland: the look, components and key bindings of Omarchy
# 3.8.4 (https://github.com/basecamp/omarchy, MIT, see ./LICENSE) for NixOS.
# The configuration lives in /etc/xdg/omarchy and is used only by this
# session; its components are user units started with the session.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.calamares.omarchy;
  configDir = "/etc/xdg/omarchy";
  # Content-addressed source and lock: the installation media prebuild this
  # from a different directory than the installed /etc/nixos, and both must
  # produce the same derivation.
  toolSource = builtins.path {
    path = ./tool;
    name = "calamares-omarchy-tool-src";
    filter = path: _: baseNameOf path != "target";
  };
  tool = pkgs.rustPlatform.buildRustPackage {
    pname = "calamares-omarchy-tool";
    version = "0.1.0";
    src = toolSource;
    cargoLock.lockFile = "${toolSource}/Cargo.lock";
    meta = {
      description = "Session launcher and desktop commands for Omarchy-style Hyprland";
      license = lib.licenses.gpl3Plus;
      mainProgram = "omarchy";
    };
  };
  session = pkgs.writeTextFile {
    name = "omarchy-session";
    destination = "/share/wayland-sessions/omarchy.desktop";
    text = ''
      [Desktop Entry]
      Name=Omarchy-style Hyprland
      Comment=Hyprland in the style of Omarchy, managed by UWSM
      Exec=${lib.getExe config.programs.uwsm.package} start -e -D Hyprland -N Omarchy -- ${lib.getExe tool} session
      Type=Application
      DesktopNames=Hyprland
    '';
    derivationArgs.passthru.providedSessions = [ "omarchy" ];
  };
  # The installed keyboard layout, with Omarchy's compose key on Caps Lock.
  xkb = config.services.xserver.xkb;
  options = lib.concatStringsSep "," (
    lib.filter (option: option != "") [
      "compose:caps"
      xkb.options
    ]
  );
  hyprland = pkgs.writeText "omarchy-hyprland.lua" ''
    ${builtins.readFile ./config/hypr/hyprland.lua}
    -- Keyboard layout of the installed system (services.xserver.xkb).
    hl.config({
      input = {
        kb_layout = ${builtins.toJSON xkb.layout},
        kb_variant = ${builtins.toJSON xkb.variant},
        kb_options = ${builtins.toJSON options},
      },
    })
  '';
  wallpaper = pkgs.nixos-artwork.wallpapers.nineish-catppuccin-mocha.gnomeFilePath;
  files = lib.filter (file: lib.path.removePrefix ./config file != "./hypr/hyprland.lua") (
    lib.filesystem.listFilesRecursive ./config
  );
  etcFile =
    file:
    lib.nameValuePair "xdg/omarchy/${lib.removePrefix "./" (lib.path.removePrefix ./config file)}" {
      source = file;
    };
  # Session units: started with wayland-session@omarchy.target (uwsm names it
  # after the compositor command), so they never run in other desktops. A
  # target orders the units it wants before itself, and it precedes
  # graphical-session.target: order after the compositor and its exported
  # environment instead, which avoids an ordering cycle.
  unit =
    description: command:
    {
      inherit description;
      wantedBy = [ "wayland-session@omarchy.target" ];
      partOf = [ "graphical-session.target" ];
      after = [
        "wayland-wm@omarchy.service"
        "wayland-session-waitenv.service"
      ];
      enableDefaultPath = false;
      environment = {
        PATH = "/run/wrappers/bin:/etc/profiles/per-user/%u/bin:/run/current-system/sw/bin";
        XDG_CONFIG_DIRS = "${configDir}:/etc/xdg:/etc/profiles/per-user/%u/etc/xdg:/run/current-system/sw/etc/xdg";
      };
      serviceConfig = {
        ExecStart = command;
        Restart = "on-failure";
        RestartSec = 1;
      };
    };
in
{
  options.calamares.omarchy.enable = lib.mkEnableOption "Omarchy-style Hyprland session";
  config = lib.mkIf cfg.enable {
    # programs.hyprland and uwsm are enabled by ../hyprland.nix.
    services.displayManager.sessionPackages = [ session ];
    environment.etc = lib.listToAttrs (map etcFile files) // {
      "xdg/omarchy/hypr/hyprland.lua".source = hyprland;
      "xdg/omarchy/background".source = wallpaper;
    };
    systemd.user.services = {
      omarchy-waybar =
        lib.recursiveUpdate
          (unit "Omarchy-style top bar" "${lib.getExe pkgs.waybar} -c ${configDir}/waybar/config.jsonc -s ${configDir}/waybar/style.css")
          {
            # The volume module does not appear if the audio graph is not
            # ready when it connects.
            after = [
              "wayland-wm@omarchy.service"
              "wayland-session-waitenv.service"
              "wireplumber.service"
              "pipewire-pulse.service"
            ];
            wants = [ "wireplumber.service" ];
          };
      omarchy-mako = unit "Omarchy-style notifications" "${pkgs.mako}/bin/mako -c ${configDir}/mako/config";
      omarchy-swaybg = unit "Omarchy-style background" "${lib.getExe pkgs.swaybg} -i ${configDir}/background -m fill";
      omarchy-swayosd = unit "Omarchy-style on-screen display" "${pkgs.swayosd}/bin/swayosd-server";
      omarchy-hypridle = unit "Omarchy-style idle locking" "${lib.getExe pkgs.hypridle} -c ${configDir}/hypr/hypridle.conf";
      omarchy-polkit = unit "Omarchy-style authentication agent" "${pkgs.hyprpolkitagent}/libexec/hyprpolkitagent";
      omarchy-elephant = unit "Omarchy-style launcher backend" "${lib.getExe pkgs.elephant}";
      omarchy-walker =
        lib.recursiveUpdate
          (unit "Omarchy-style launcher" "${lib.getExe pkgs.walker} --gapplication-service")
          {
            after = [
              "wayland-wm@omarchy.service"
              "wayland-session-waitenv.service"
              "omarchy-elephant.service"
            ];
            wants = [ "omarchy-elephant.service" ];
            environment.GSK_RENDERER = "cairo";
          };
      # Started on demand by the nightlight toggle.
      omarchy-hyprsunset = (unit "Omarchy-style nightlight" "${lib.getExe pkgs.hyprsunset}") // {
        wantedBy = [ ];
      };
    };
    security.pam.services.hyprlock = { };
    services.power-profiles-daemon.enable = lib.mkDefault true;
    hardware.bluetooth.enable = lib.mkDefault true;
    # Terminal applications started from the launcher open in Alacritty in
    # Hyprland sessions; other desktops keep their own terminal.
    xdg.terminal-exec = {
      enable = true;
      settings.Hyprland = [ "Alacritty.desktop" ];
    };
    # Dark GTK applications with Yaru icons, as Omarchy's Tokyo Night theme
    # sets them. These are defaults: a user's own settings take precedence.
    programs.dconf.profiles.user.databases = [
      {
        settings."org/gnome/desktop/interface" = {
          color-scheme = "prefer-dark";
          gtk-theme = "Adwaita-dark";
          icon-theme = "Yaru-magenta";
        };
      }
    ];
    fonts.packages = [
      pkgs.nerd-fonts.jetbrains-mono
      pkgs.font-awesome
      pkgs.noto-fonts-color-emoji
    ];
    environment.systemPackages = [
      tool
      pkgs.alacritty
      pkgs.walker
      # walker finds its backend through PATH.
      pkgs.elephant
      pkgs.mako
      pkgs.swayosd
      pkgs.hyprlock
      pkgs.hyprpicker
      pkgs.hyprsunset
      pkgs.grim
      pkgs.slurp
      pkgs.satty
      pkgs.wl-clipboard
      pkgs.libnotify
      pkgs.brightnessctl
      pkgs.playerctl
      pkgs.wireplumber
      pkgs.wiremix
      pkgs.bluetui
      pkgs.btop
      pkgs.fastfetch
      pkgs.nautilus
      pkgs.imv
      pkgs.mpv
      pkgs.gnome-calculator
      pkgs.xdg-utils
      pkgs.yaru-theme
      pkgs.gnome-themes-extra
      pkgs.adwaita-icon-theme
    ];
  };
}
