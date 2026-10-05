// SPDX-License-Identifier: GPL-3.0-or-later
//! `omarchy`: session launcher and desktop commands for the Omarchy-style
//! Hyprland session, replacing the Bash `omarchy-*` scripts it is modelled
//! on (https://github.com/basecamp/omarchy, MIT).
mod browser;
mod capture;
mod hypr;
mod media;
mod menu;
mod power;
mod terminal;
mod toggle;
mod util;

use std::{path::Path, process::ExitCode};

/// Omarchy-style configuration shipped by the NixOS module.
pub const CONFIG: &str = "/etc/xdg/omarchy";

const USAGE: &str = "usage: omarchy <command>
  session                         start Hyprland with the Omarchy-style configuration
  menu [main|system|capture|toggle|setup|power]
  keybindings | about
  terminal [command...]           terminal in the focused terminal's directory
  tui <command...>                floating terminal application
  browser [--private] [url...]
  volume <raise|lower|mute-toggle|+N|-N> | mic-mute
  brightness <+5%|5%-|N%|on|off>  | media <next|previous|play-pause>
  screenshot [smart|region|windows|fullscreen] | color-picker
  lock [--lock-only] | wake [seconds] | logout | reboot | shutdown | close-all
  toggle <idle|nightlight|notifications|bar> | indicator <idle|notifications>";

/// XDG_CONFIG_DIRS with the Omarchy directory first, without duplicates.
pub fn config_dirs(existing: Option<&str>) -> String {
    let mut dirs = vec![CONFIG];
    dirs.extend(
        existing
            .unwrap_or("/etc/xdg")
            .split(':')
            .filter(|dir| !dir.is_empty() && *dir != CONFIG),
    );
    dirs.join(":")
}

/// Start the compositor. A user's own ~/.config/omarchy/hyprland.lua
/// replaces the shipped configuration entirely.
fn session() -> ExitCode {
    let user = format!("{}/.config/omarchy/hyprland.lua", util::home());
    let config = if Path::new(&user).is_file() {
        user
    } else {
        format!("{CONFIG}/hypr/hyprland.lua")
    };
    let launcher = util::which("start-hyprland")
        .and_then(|path| path.to_str().map(str::to_owned))
        .unwrap_or_else(|| "/run/current-system/sw/bin/start-hyprland".into());
    let mut command = std::process::Command::new(&launcher);
    command
        .env(
            "XDG_CONFIG_DIRS",
            config_dirs(std::env::var("XDG_CONFIG_DIRS").ok().as_deref()),
        )
        .args(["--", "--config", &config]);
    let error = std::os::unix::process::CommandExt::exec(&mut command);
    eprintln!("omarchy: cannot start {launcher}: {error}");
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or_default();
    let first = |default: &'static str| rest.first().map_or(default, String::as_str);
    match args.first().map(String::as_str) {
        Some("session") => return session(),
        Some("menu") => menu::show(first("main")),
        Some("keybindings") => menu::keybindings(),
        Some("about") => terminal::tui(
            &[
                "fastfetch".into(),
                "--config".into(),
                format!("{CONFIG}/fastfetch/config.jsonc"),
            ],
            true,
        ),
        Some("terminal") => terminal::terminal(rest),
        Some("tui") if !rest.is_empty() => terminal::tui(rest, false),
        Some("browser") => browser::launch(rest),
        Some("volume") => media::volume(first("raise")),
        Some("mic-mute") => media::mic_mute(),
        Some("brightness") => media::brightness(first("+5%")),
        Some("media") => media::media(first("play-pause")),
        Some("screenshot") => capture::screenshot(first("smart")),
        Some("screenshot-notify") if rest.len() == 1 => capture::notify_edit(&rest[0]),
        Some("color-picker") => capture::color_picker(),
        Some("lock") => power::lock(rest.iter().any(|arg| arg == "--lock-only")),
        Some("lock-wait") => power::lock_wait(),
        Some("wake") => power::wake(first("0").parse().unwrap_or(0)),
        Some("close-all") => power::close_all(),
        Some("logout") => power::leave(power::Leave::Logout),
        Some("reboot") => power::leave(power::Leave::Reboot),
        Some("shutdown") => power::leave(power::Leave::Shutdown),
        Some("toggle") => match first("") {
            "idle" => toggle::idle(),
            "nightlight" => toggle::nightlight(),
            "notifications" => toggle::notifications(),
            "bar" => toggle::bar(),
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::FAILURE;
            }
        },
        Some("indicator") => println!("{}", toggle::indicator(first(""))),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn omarchy_configuration_is_searched_first_once() {
        assert_eq!(config_dirs(None), "/etc/xdg/omarchy:/etc/xdg");
        assert_eq!(
            config_dirs(Some(
                "/etc/xdg:/etc/xdg/omarchy::/run/current-system/sw/etc/xdg"
            )),
            "/etc/xdg/omarchy:/etc/xdg:/run/current-system/sw/etc/xdg"
        );
    }
}
