# Tatami

Tatami is a keyboard-driven [Hyprland](https://hypr.land) desktop for NixOS,
inspired by [Omarchy](https://github.com/basecamp/omarchy) 4.0.4 ("Quattro"):
its look, behavior and key bindings, adapted for NixOS without home-manager.
It is not Omarchy and is not affiliated with Omarchy or Basecamp.

Omarchy 4 draws its bar, menus, notifications, OSD and lock screen with a
Quickshell (QML/JavaScript) shell driven by Bash. Tatami reproduces that design
with Waybar, Walker, Mako, SwayOSD and hyprlock, styled to match, and replaces
the scripts with one Rust program, `tatami`.

It is one of the desktops of the
[NixOS graphical installer](https://github.com/bitemyapp/determinate-nixos-graphical),
which installs it through its [installer modules](https://github.com/bitemyapp/calamares),
and it works in any NixOS flake on its own.

## Using it

```nix
{
  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
  inputs.tatami = {
    url = "github:bitemyapp/tatami/stable";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { nixpkgs, tatami, ... }: {
    nixosConfigurations.example = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        tatami.nixosModules.default
        { programs.tatami.enable = true; }
        ./configuration.nix
      ];
    };
  };
}
```

`programs.tatami.enable` adds a **Tatami** session to the login screen and
enables Hyprland with [uwsm](https://github.com/Vladimir-csp/uwsm), which it
needs. Tatami needs a display manager that offers Wayland sessions, such as
Plasma Login Manager, SDDM or GDM, and NetworkManager for its Wi-Fi panel.

`nix flake update tatami` and a rebuild bring Tatami's updates. The `stable`
branch has verified releases; `main` is reviewed work.

## What it is

| Component | Configuration |
|---|---|
| Look | Tokyo Night throughout, JetBrainsMono Nerd Font. Window gaps 5/10, a 2px `#7aa2f7` border, square corners, no blur or shadow, v4 animations, inactive windows at 0.985/0.96 opacity. NixOS artwork wallpaper, since Omarchy's wallpapers have unclear licenses |
| Bar | Waybar, 26px. NixOS logo (menu), workspaces 1–5, a centered "Monday 10:23" clock with night-light, do-not-disturb and stay-awake indicators, then tray, Bluetooth, network, audio, display and battery |
| Menu and launcher | Walker with Elephant. The v4 menu tree (Apps, Learn, Trigger, Setup, About, System) in v4's card style. Typing searches menu entries and applications together |
| Notifications and OSD | Mako as v4's 380px accent-bordered cards; SwayOSD as v4's bottom card |
| Lock and idle | hyprlock with PAM in v4's style; hypridle locks and turns off displays |
| Panels | Wi-Fi as a Walker list over NetworkManager: networks by signal, a password prompt for new ones, the Wi-Fi switch and Network settings… (`nm-connection-editor`) for VPN, enterprise and hidden networks. The bar's network icon, Super+Ctrl+W and Setup › Wi-Fi all open it. Bluetooth is bluetui (it pairs devices that ask for a passkey), audio is wiremix |
| Terminal and tools | foot with v4's configuration, btop with the Tokyo Night theme, fastfetch, Nautilus, imv, mpv |
| GTK applications | Dark Adwaita with Yaru icons, from a dconf profile selected only in this session |

The configuration lives in `/etc/xdg/tatami` and is used only by this
session. The session puts that directory first in `XDG_CONFIG_DIRS` and
starts Hyprland with `--config`, so other desktops on the same system are
unaffected. Its components are systemd user units wanted by the session's own
target, so they never start elsewhere. `config/hypr/hyprland.lua` contains
only declarative `hl.*` calls: v4's window rules and key bindings, minus
Arch-specific ones. Both Shift keys together toggle Caps Lock. The module
appends the system's keyboard layout (`services.xserver.xkb`); non-Latin
layouts follow `us`, switched with both Alt keys.

The `tatami` helper (`tool/`) provides:

- the session launcher
- the menu: system (lock, suspend, hibernate when swap exists, logout,
  restart, shutdown), capture, toggles, hardware, display, audio, Wi-Fi,
  Bluetooth and power profile; the power key opens System, as in Omarchy
- the Wi-Fi list; passwords reach NetworkManager in a file only the user can
  read, never on a command line, and are stored root-only
- key binding help
- copy and paste that works in terminals (Super+C/V/X)
- window commands: pop out, tiled fullscreen, transparency, gaps, square
  aspect, workspace layout. Gaps, aspect, layouts, display scaling and a
  disabled touchpad are re-applied at login
- monitor scaling, laptop display, touchpad and zoom toggles
- time, battery and keyboard backlight notifications
- a terminal in the active window's directory, editor and browser launch
- volume, microphone, brightness and media keys with OSD
- screenshots (freeze, region or window, copy, notify, Satty editing)
- lock and wake, and logout, reboot and shutdown after closing windows

Applications started from the menu, launcher or key bindings run in their own
systemd scopes, so a memory-pressure kill of one cannot end the session.

With other desktops installed, Tatami keeps their pieces out: their tray
applets (nm-applet, the printer applet) do not start in it, and the session
reports `XDG_CURRENT_DESKTOP=Hyprland:Tatami` so such entries can tell it
apart. Setup › Hyprland creates `~/.config/tatami/hyprland.lua`, which loads
Tatami's defaults first and then your own settings, so later updates of the
defaults still apply.

Not ported from Omarchy:

- Arch package installation and updates, Limine/snapper, Plymouth theming and
  the first-run script.
- Theme, background and font switching, web apps, screen recording, OCR,
  sharing, reminders, weather, dictation and AI agents.
- The shell's other drop-down panels: Bluetooth and audio open bluetui and
  wiremix. There is no notification history panel.

## Development

`nix flake check` builds the helper (with its tests) and evaluates an example
system with the module. The helper is a Cargo crate in `tool/`; `cargo test`
there runs its tests on Linux.

New code is Rust; configuration stays declarative (Nix, TOML, JSONC, CSS and
`hl.*` calls in `hyprland.lua`).

## License

Tatami is licensed under either of

- the [Apache License, Version 2.0](LICENSE-APACHE), or
- the [MIT License](LICENSE-MIT),

at your option. The files under `config/` are adapted from Omarchy, which is
MIT-licensed; its copyright notice is in [config/LICENSE](config/LICENSE).

Unless you state otherwise, any contribution you intentionally submit for
inclusion in Tatami, as defined in the Apache-2.0 license, is dual licensed
as above, without additional terms or conditions.
