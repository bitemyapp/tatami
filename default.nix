# SPDX-License-Identifier: GPL-3.0-or-later
# Tatami: the look, behavior and key bindings of Omarchy 4
# "Quattro" (v4.0.4, https://github.com/basecamp/omarchy, MIT, see ./LICENSE)
# for NixOS. Omarchy 4's Quickshell desktop shell is replaced by Waybar,
# Walker/Elephant, Mako, SwayOSD and hyprlock styled after it, and its Bash
# commands by the Rust `tatami` helper. The configuration lives in
# /etc/xdg/tatami and is used only by this session; its components are user
# units started with the session.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.calamares.tatami;
  configDir = "/etc/xdg/tatami";
  tool = pkgs.callPackage ./tool/package.nix { };
  # Only the providers the launcher and menu use (../config/walker): not
  # Arch package search, password managers or other compositors' windows.
  elephant = pkgs.elephant.override {
    enabledProviders = [
      "desktopapplications"
      "menus"
      "providerlist"
      "calc"
      "clipboard"
      "files"
      "symbols"
      "websearch"
    ];
  };
  # Other desktops' tray applets, kept out of this session: Tatami has its own
  # Wi-Fi list, and the bar shows no printer applet.
  hiddenInTatami =
    name: package: notShowIn:
    lib.nameValuePair "xdg/autostart/${name}.desktop" {
      source = pkgs.substitute {
        name = "${name}.desktop";
        src = "${package}/etc/xdg/autostart/${name}.desktop";
        substitutions = [
          "--replace-fail"
          "NotShowIn=${notShowIn}"
          "NotShowIn=${notShowIn}Tatami;"
        ];
      };
    };
  # Desktop names Hyprland:Tatami: Hyprland first, so portals and the
  # terminal choice follow Hyprland's; Tatami makes uwsm load env-tatami.
  session = pkgs.writeTextFile {
    name = "tatami-session";
    destination = "/share/wayland-sessions/tatami.desktop";
    text = ''
      [Desktop Entry]
      Name=Tatami
      Comment=Keyboard-driven Hyprland inspired by Omarchy, managed by UWSM
      Exec=${lib.getExe config.programs.uwsm.package} start -e -D Hyprland:Tatami -N Tatami -- ${lib.getExe tool} session
      Type=Application
      DesktopNames=Hyprland;Tatami
    '';
    derivationArgs.passthru.providedSessions = [ "tatami" ];
  };
  # The installed keyboard layout (services.xserver.xkb) with Omarchy's
  # options: the compose key on Caps Lock, and Caps Lock on both Shifts
  # (released by the next lone Shift). Hyprland resolves key bindings against
  # the first layout, so a layout that cannot type Latin letters goes after
  # "us" and is reached with both Alts (default/hypr/input.lua).
  xkb = config.services.xserver.xkb;
  nonLatin = [
    "af"
    "am"
    "ara"
    "bd"
    "bg"
    "by"
    "et"
    "ge"
    "gr"
    "il"
    "in"
    "iq"
    "ir"
    "kg"
    "kh"
    "kz"
    "la"
    "lk"
    "mk"
    "mm"
    "mn"
    "mv"
    "np"
    "rs"
    "ru"
    "sy"
    "th"
    "tj"
    "ua"
  ];
  latinFirst = !lib.elem (lib.head (lib.splitString "," xkb.layout)) nonLatin;
  options = lib.concatStringsSep "," (
    lib.filter (option: option != "") (
      [
        "compose:caps"
        "shift:both_capslock_cancel"
        xkb.options
      ]
      ++ lib.optional (!latinFirst) "grp:alts_toggle"
    )
  );
  hyprland = pkgs.writeText "tatami-hyprland.lua" ''
    ${builtins.readFile ./config/hypr/hyprland.lua}
    -- Keyboard layout of the installed system (services.xserver.xkb).
    hl.config({
      input = {
        kb_layout = ${builtins.toJSON (if latinFirst then xkb.layout else "us,${xkb.layout}")},
        kb_variant = ${builtins.toJSON (if latinFirst then xkb.variant else ",${xkb.variant}")},
        kb_options = ${builtins.toJSON options},
      },
    })
  '';
  # System › in the menu, with Hibernate only when the installed system can
  # resume from a swap partition (Omarchy checks this at run time). Icons are
  # TOML escapes for the Nerd Font glyphs Omarchy uses.
  systemEntry = text: icon: value: ''

    [[entries]]
    text = "${text}"
    icon = "${icon}"
    value = "${value}"
  '';
  systemMenu = pkgs.writeText "tatami-system.toml" (
    ''
      # System › in the Tatami menu (see tatami.toml), written by NixOS.
      name = "tatami-system"
      name_pretty = "System"
      parent = "tatami"
      fixed_order = true
      hide_from_providerlist = true
      action = "%VALUE%"
    ''
    + systemEntry "Lock" "\\uF023" "tatami lock"
    + systemEntry "Suspend" "\\U000F04B2" "tatami suspend"
    + lib.optionalString (config.boot.resumeDevice != "") (
      systemEntry "Hibernate" "\\U000F0901" "tatami hibernate"
    )
    + systemEntry "Logout" "\\U000F0343" "tatami logout"
    + systemEntry "Reboot" "\\U000F0709" "tatami reboot"
    + systemEntry "Shutdown" "\\U000F0425" "tatami shutdown"
  );
  wallpaper = pkgs.nixos-artwork.wallpapers.nineish-catppuccin-mocha.gnomeFilePath;
  files = lib.filter (file: lib.path.removePrefix ./config file != "./hypr/hyprland.lua") (
    lib.filesystem.listFilesRecursive ./config
  );
  etcFile =
    file:
    lib.nameValuePair "xdg/tatami/${lib.removePrefix "./" (lib.path.removePrefix ./config file)}" {
      source = file;
    };
  # Session units: started with wayland-session@tatami.target (uwsm names it
  # after the compositor command), so they never run in other desktops. A
  # target orders the units it wants before itself, and it precedes
  # graphical-session.target: order after the compositor and its exported
  # environment instead, which avoids an ordering cycle.
  unit = description: command: {
    inherit description;
    wantedBy = [ "wayland-session@tatami.target" ];
    partOf = [ "graphical-session.target" ];
    after = [
      "wayland-wm@tatami.service"
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
  options.calamares.tatami.enable = lib.mkEnableOption "Tatami session";
  config = lib.mkIf cfg.enable {
    # programs.hyprland and uwsm are enabled by ../hyprland.nix.
    services.displayManager.sessionPackages = [ session ];
    environment.etc =
      lib.listToAttrs (map etcFile files)
      // {
        "xdg/tatami/hypr/hyprland.lua".source = hyprland;
        "xdg/tatami/elephant/menus/tatami-system.toml".source = systemMenu;
        "xdg/tatami/background".source = wallpaper;
        # Selects the session's dconf profile (see programs.dconf below).
        "xdg/uwsm/env-tatami".text = ''
          export DCONF_PROFILE=tatami
        '';
      }
      // lib.listToAttrs (
        [ (hiddenInTatami "nm-applet" pkgs.networkmanagerapplet "KDE;GNOME;COSMIC;") ]
        ++ lib.optional config.programs.system-config-printer.enable (
          hiddenInTatami "print-applet" pkgs.system-config-printer "KDE;GNOME;Cinnamon;"
        )
      );
    systemd.user.services = {
      tatami-waybar =
        lib.recursiveUpdate
          (unit "Tatami top bar" "${lib.getExe pkgs.waybar} -c ${configDir}/waybar/config.jsonc -s ${configDir}/waybar/style.css")
          {
            # The volume module does not appear if the audio graph is not
            # ready when it connects.
            after = [
              "wayland-wm@tatami.service"
              "wayland-session-waitenv.service"
              "wireplumber.service"
              "pipewire-pulse.service"
            ];
            wants = [ "wireplumber.service" ];
          };
      # A plain service: with Type=dbus and BusName, systemd refused to load it
      # once another desktop's notification service had claimed the name.
      tatami-mako = unit "Tatami notifications" "${pkgs.mako}/bin/mako -c ${configDir}/mako/config";
      # The power key opens the System menu, as in Omarchy, which has logind
      # ignore it; here only while this session runs, so the login screen and
      # the other desktops keep their own handling.
      tatami-power-key = unit "Tatami power key" (
        lib.escapeShellArgs [
          "${config.systemd.package}/bin/systemd-inhibit"
          "--what=handle-power-key"
          "--who=Tatami"
          "--why=The power key opens the System menu"
          "--mode=block"
          "${pkgs.coreutils}/bin/sleep"
          "infinity"
        ]
      );
      tatami-swaybg = unit "Tatami background" "${lib.getExe pkgs.swaybg} -i ${configDir}/background -m fill";
      tatami-swayosd = unit "Tatami on-screen display" "${pkgs.swayosd}/bin/swayosd-server";
      tatami-hypridle = unit "Tatami idle locking" "${lib.getExe pkgs.hypridle} -c ${configDir}/hypr/hypridle.conf";
      tatami-polkit = unit "Tatami authentication agent" "${pkgs.hyprpolkitagent}/libexec/hyprpolkitagent";
      tatami-elephant = unit "Tatami launcher backend" "${lib.getExe elephant}";
      tatami-walker =
        lib.recursiveUpdate (unit "Tatami launcher" "${lib.getExe pkgs.walker} --gapplication-service")
          {
            after = [
              "wayland-wm@tatami.service"
              "wayland-session-waitenv.service"
              "tatami-elephant.service"
            ];
            wants = [ "tatami-elephant.service" ];
            environment.GSK_RENDERER = "cairo";
          };
      # Removable drives mount automatically, as in Omarchy 4.
      tatami-udiskie = unit "Tatami automounting" "${lib.getExe' pkgs.udiskie "udiskie"} --automount --no-notify --no-tray";
      # Window gaps, aspect ratio, workspace layouts, monitor scales and a
      # disabled touchpad, as the user last toggled them.
      tatami-restore =
        lib.recursiveUpdate (unit "Tatami remembered toggles" "${lib.getExe tool} restore")
          {
            serviceConfig = {
              Type = "oneshot";
              Restart = "no";
            };
          };
      # Started on demand by the nightlight toggle.
      tatami-hyprsunset = (unit "Tatami nightlight" "${lib.getExe pkgs.hyprsunset}") // {
        wantedBy = [ ];
      };
    };
    security.pam.services.hyprlock = { };
    services.power-profiles-daemon.enable = lib.mkDefault true;
    services.udisks2.enable = lib.mkDefault true;
    hardware.bluetooth.enable = lib.mkDefault true;
    # Terminal applications open in foot, Omarchy 4's terminal, in Hyprland
    # sessions; other desktops keep their own terminal.
    xdg.terminal-exec = {
      enable = true;
      settings.Hyprland = [ "foot.desktop" ];
    };
    # Dark GTK applications with Yaru icons, as Omarchy's Tokyo Night theme
    # sets them, in this session only. uwsm reads /etc/xdg/uwsm/env-tatami when
    # preparing the session, exports DCONF_PROFILE to the user manager (and so
    # to applications, the GTK portal and session units) and removes it at
    # logout, so other desktops keep their own defaults even when the user
    # manager lingers. These are defaults: the user's own settings (user-db)
    # take precedence.
    programs.dconf.profiles.tatami.databases = [
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
      # Notification text (Omarchy 4's notification cards).
      pkgs.liberation_ttf
    ];
    environment.systemPackages = [
      tool
      pkgs.foot
      pkgs.walker
      # walker finds its backend through PATH.
      elephant
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
      # Network settings… in the Wi-Fi list: VPN, enterprise and hidden
      # networks, static addresses.
      pkgs.networkmanagerapplet
      pkgs.btop
      pkgs.fastfetch
      pkgs.nautilus
      pkgs.imv
      pkgs.mpv
      pkgs.gnome-calculator
      pkgs.xdg-utils
      pkgs.udiskie
      pkgs.yaru-theme
      pkgs.gnome-themes-extra
      pkgs.adwaita-icon-theme
    ];
  };
}
