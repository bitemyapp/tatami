// SPDX-License-Identifier: GPL-3.0-or-later
//! `omarchy`: session launcher and desktop commands for the Omarchy-style
//! Hyprland session, replacing the Bash `omarchy-*` scripts of Omarchy 4
//! (https://github.com/basecamp/omarchy, MIT) that it is modelled on.
mod browser;
mod capture;
mod clip;
mod hypr;
mod info;
mod media;
mod menu;
mod power;
mod terminal;
mod toggle;
mod util;
mod window;

use std::{path::Path, process::ExitCode};

/// Omarchy-style configuration shipped by the NixOS module.
pub const CONFIG: &str = "/etc/xdg/omarchy";

const USAGE: &str = "usage: omarchy <command>
  session                         start Hyprland with the Omarchy-style configuration
  restore                         apply remembered toggles (session start)
  menu [learn|trigger|capture|toggle|hardware|setup|display|system]
  apps | emoji | keybindings | about | power-profile | edit-config
  terminal [command...]           terminal in the focused terminal's directory
  tui <command...>                terminal application (focuses an open one)
  editor [file...] | files [--cwd] | launch <program> [args...]
  browser [--private] [url...]
  volume <raise|lower|mute-toggle|+N|-N> | mic-mute
  brightness <+5%|5%-|N%|+1%|1%-> | keyboard-brightness <up|down|cycle>
  media <next|previous|play-pause>
  clipboard <copy|paste|cut>
  screenshot [smart|region|windows|fullscreen] | color-picker
  window <pop|tiled-fullscreen|transparency|close-all>
  scale <up|down> | laptop-display | touchpad [on|off|toggle] | zoom <in|reset>
  toggle <idle|nightlight|notifications|bar|gaps|aspect|layout>
  indicator <idle|notifications|nightlight>
  notify <time|battery> | bluetooth-toggle
  lock [--lock-only] | wake [seconds] | logout | reboot | shutdown | suspend | hibernate";

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

/// Setup › Hyprland: edit the user's own copy of the configuration, made
/// from the shipped one the first time. The session uses it from the next
/// login on.
fn edit_config() {
    use std::os::unix::fs::PermissionsExt;
    let dir = format!("{}/.config/omarchy", util::home());
    let user = format!("{dir}/hyprland.lua");
    if !Path::new(&user).exists() {
        let copied = std::fs::create_dir_all(&dir).is_ok()
            && std::fs::copy(format!("{CONFIG}/hypr/hyprland.lua"), &user).is_ok()
            && std::fs::set_permissions(&user, std::fs::Permissions::from_mode(0o644)).is_ok();
        if !copied {
            util::notify(
                "\u{f359}",
                "Cannot create ~/.config/omarchy/hyprland.lua",
                "",
            );
            return;
        }
        util::notify(
            "\u{f359}",
            "Your Hyprland configuration",
            "~/.config/omarchy/hyprland.lua replaces the default from the next login",
        );
    }
    terminal::editor(&[user]);
}

/// Super+Shift+F (and Super+Alt+Shift+F in the terminal's directory).
fn files(cwd: bool) {
    let dir = cwd.then(terminal::active_cwd);
    let mut args = vec!["--new-window"];
    if let Some(dir) = &dir {
        args.push(dir);
    }
    util::launch("nautilus", &args);
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or_default();
    let first = |default: &'static str| rest.first().map_or(default, String::as_str);
    let ok = match args.first().map(String::as_str) {
        Some("session") => return session(),
        Some("restore") => {
            window::restore();
            true
        }
        Some("menu") => {
            menu::show(first(""));
            menu::walker_args(first("")).is_some()
        }
        Some("apps") => {
            menu::apps();
            true
        }
        Some("emoji") => {
            menu::emoji();
            true
        }
        Some("edit-config") => {
            edit_config();
            true
        }
        Some("keybindings") => {
            menu::keybindings();
            true
        }
        Some("power-profile") => {
            menu::power_profile();
            true
        }
        Some("about") => {
            terminal::tui(
                &[
                    "fastfetch".into(),
                    "--config".into(),
                    format!("{CONFIG}/fastfetch/config.jsonc"),
                ],
                terminal::Tui::Hold,
            );
            true
        }
        Some("terminal") => {
            terminal::terminal(rest);
            true
        }
        Some("tui") if !rest.is_empty() => {
            terminal::tui(rest, terminal::Tui::Focus);
            true
        }
        Some("editor") => {
            terminal::editor(rest);
            true
        }
        Some("files") => {
            files(rest.iter().any(|arg| arg == "--cwd"));
            true
        }
        Some("launch") if !rest.is_empty() => {
            let args: Vec<&str> = rest[1..].iter().map(String::as_str).collect();
            util::launch(&rest[0], &args);
            true
        }
        Some("browser") => {
            browser::launch(rest);
            true
        }
        Some("volume") => {
            media::volume(first("raise"));
            true
        }
        Some("mic-mute") => {
            media::mic_mute();
            true
        }
        Some("brightness") => {
            media::brightness(first("+5%"));
            true
        }
        Some("keyboard-brightness") => {
            media::keyboard_brightness(first("cycle"));
            true
        }
        Some("media") => {
            media::media(first("play-pause"));
            true
        }
        Some("clipboard") => {
            clip::send(first(""));
            clip::chord(first(""), false).is_some()
        }
        Some("screenshot") => {
            capture::screenshot(first("smart"));
            true
        }
        Some("screenshot-notify") if rest.len() == 1 => {
            capture::notify_edit(&rest[0]);
            true
        }
        Some("color-picker") => {
            capture::color_picker();
            true
        }
        Some("window") => match first("") {
            "pop" => {
                window::pop();
                true
            }
            "tiled-fullscreen" => {
                window::tiled_fullscreen();
                true
            }
            "transparency" => {
                window::transparency();
                true
            }
            "close-all" => {
                window::close_all();
                true
            }
            _ => false,
        },
        Some("scale") => {
            window::scale(first("up") == "up");
            true
        }
        Some("laptop-display") => {
            window::laptop_display();
            true
        }
        Some("touchpad") => {
            window::touchpad(first("toggle"));
            true
        }
        Some("zoom") => {
            window::zoom(first("in"));
            true
        }
        Some("toggle") => match first("") {
            "idle" => {
                toggle::idle();
                true
            }
            "nightlight" => {
                toggle::nightlight();
                true
            }
            "notifications" => {
                toggle::notifications();
                true
            }
            "bar" => {
                toggle::bar();
                true
            }
            "gaps" => {
                window::gaps();
                true
            }
            "aspect" => {
                window::aspect();
                true
            }
            "layout" => {
                window::workspace_layout();
                true
            }
            _ => false,
        },
        Some("indicator") => {
            println!("{}", toggle::indicator(first("")));
            true
        }
        Some("notify") => match first("") {
            "time" => {
                info::time();
                true
            }
            "battery" => {
                info::battery();
                true
            }
            _ => false,
        },
        Some("bluetooth-toggle") => {
            power::bluetooth_toggle();
            true
        }
        Some("lock") => {
            power::lock(rest.iter().any(|arg| arg == "--lock-only"));
            true
        }
        Some("lock-wait") => {
            power::lock_wait();
            true
        }
        Some("wake") => {
            power::wake(first("0").parse().unwrap_or(0));
            true
        }
        Some("logout") => {
            power::leave(power::Leave::Logout);
            true
        }
        Some("reboot") => {
            power::leave(power::Leave::Reboot);
            true
        }
        Some("shutdown") => {
            power::leave(power::Leave::Shutdown);
            true
        }
        Some("suspend") => {
            util::run("systemctl", &["suspend"]);
            true
        }
        Some("hibernate") => {
            power::hibernate();
            true
        }
        _ => false,
    };
    if ok {
        ExitCode::SUCCESS
    } else {
        eprintln!("{USAGE}");
        ExitCode::FAILURE
    }
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
