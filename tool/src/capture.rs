// SPDX-License-Identifier: GPL-3.0-or-later
//! Screenshots: freeze the screen, select with slurp, save, copy, and offer
//! editing in Satty from the notification.
use crate::{
    hypr::{self, Rect},
    util,
};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, SystemTime},
};

/// XDG_PICTURES_DIR from user-dirs.dirs (`"$HOME/Pictures"` form), else ~/Pictures.
pub fn pictures_dir(user_dirs: &str, home: &str) -> PathBuf {
    user_dirs
        .lines()
        .filter_map(|line| line.trim().strip_prefix("XDG_PICTURES_DIR="))
        .map(|value| value.trim_matches('"').replace("$HOME", home))
        .find(|value| value.starts_with('/'))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(home).join("Pictures"))
}

/// A click (a selection under 20 square pixels) means "the window or monitor
/// under the pointer", not a 2-pixel screenshot.
pub fn smart_selection(selection: Rect, candidates: &[Rect]) -> Rect {
    if selection.w * selection.h >= 20 {
        return selection;
    }
    // Windows are listed after monitors; prefer the most specific match.
    candidates
        .iter()
        .rev()
        .find(|rect| rect.contains(selection.x, selection.y))
        .copied()
        .unwrap_or(selection)
}

/// UTC timestamp for file names, without external date tools.
pub fn timestamp(seconds: u64) -> String {
    let days = seconds / 86400;
    let (hour, minute, second) = (seconds % 86400 / 3600, seconds % 3600 / 60, seconds % 60);
    // Civil-from-days (Howard Hinnant), valid for all dates after 1970.
    let z = days as i64 + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}_{hour:02}-{minute:02}-{second:02}")
}

pub fn screenshot(mode: &str) {
    // Pressing the key again while selecting cancels the selection.
    let selecting = util::pids_named(&["slurp"]);
    if !selecting.is_empty() {
        for pid in selecting {
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
        }
        return;
    }
    let home = util::home();
    let dir = pictures_dir(
        &fs::read_to_string(format!("{home}/.config/user-dirs.dirs")).unwrap_or_default(),
        &home,
    );
    if fs::create_dir_all(&dir).is_err() {
        util::notify(
            "Screenshot failed",
            &format!("Cannot create {}", dir.display()),
        );
        return;
    }
    let monitors = hypr::json("monitors").unwrap_or_default();
    let clients = hypr::json("clients").unwrap_or_default();
    let candidates = hypr::workspace_rects(&monitors, &clients);
    let selection = if mode == "fullscreen" {
        hypr::focused_monitor_rect(&monitors)
    } else {
        // Freeze the screen so the selection matches what is captured.
        let freeze = Command::new("hyprpicker")
            .args(["-r", "-z"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok();
        thread::sleep(Duration::from_millis(100));
        let picked = match mode {
            "region" => util::output("slurp", &[]),
            _ => {
                let input: String = candidates.iter().map(|r| r.slurp() + "\n").collect();
                let args: &[&str] = if mode == "windows" { &["-r"] } else { &[] };
                util::filter("slurp", args, input.as_bytes())
            }
        };
        let rect = picked.ok().and_then(|text| Rect::parse(&text)).map(|rect| {
            if mode == "region" {
                rect
            } else {
                smart_selection(rect, &candidates)
            }
        });
        let shot = rect.and_then(|rect| capture(&dir, rect));
        if let Some(mut freeze) = freeze {
            let _ = freeze.kill();
            let _ = freeze.wait();
        }
        if let Some(path) = shot {
            announce(&path);
        }
        return;
    };
    if let Some(path) = selection.and_then(|rect| capture(&dir, rect)) {
        announce(&path);
    }
}

fn capture(dir: &std::path::Path, rect: Rect) -> Option<PathBuf> {
    let seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = dir.join(format!("screenshot-{}.png", timestamp(seconds)));
    let target = path.to_str()?;
    util::run("grim", &["-g", &rect.slurp(), target]).then_some(path)
}

fn announce(path: &std::path::Path) {
    if let Ok(bytes) = fs::read(path) {
        let _ = util::filter("wl-copy", &["--type", "image/png"], &bytes);
    }
    // The notification waits for a click; let it outlive this command.
    if let Some(path) = path.to_str()
        && let Ok(me) = std::env::current_exe()
        && let Some(me) = me.to_str()
    {
        util::spawn(me, &["screenshot-notify", path]);
    }
}

/// Offer editing from the notification (or Super+Alt+, which invokes it).
pub fn notify_edit(path: &str) {
    let action = util::output(
        "notify-send",
        &[
            "Screenshot saved to clipboard and file",
            "Edit with Super + Alt + , (or click this)",
            "-t",
            "10000",
            "-i",
            path,
            "-A",
            "default=edit",
        ],
    )
    .unwrap_or_default();
    if action.trim() == "default" {
        util::run(
            "satty",
            &[
                "--filename",
                path,
                "--output-filename",
                path,
                "--actions-on-enter",
                "save-to-clipboard",
                "--save-after-copy",
                "--copy-command",
                "wl-copy",
            ],
        );
    }
}

pub fn color_picker() {
    let running = util::pids_named(&["hyprpicker"]);
    if running.is_empty() {
        util::spawn("hyprpicker", &["-a"]);
    } else {
        for pid in running {
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pictures_directory_from_user_dirs() {
        assert_eq!(
            pictures_dir("XDG_PICTURES_DIR=\"$HOME/Bilder\"\n", "/home/a"),
            PathBuf::from("/home/a/Bilder")
        );
        assert_eq!(
            pictures_dir("", "/home/a"),
            PathBuf::from("/home/a/Pictures")
        );
        assert_eq!(
            pictures_dir("XDG_PICTURES_DIR=\"relative\"", "/home/a"),
            PathBuf::from("/home/a/Pictures")
        );
    }
    #[test]
    fn clicks_select_the_window_under_the_pointer() {
        let monitor = Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
        };
        let window = Rect {
            x: 100,
            y: 100,
            w: 800,
            h: 600,
        };
        let click = Rect {
            x: 200,
            y: 200,
            w: 1,
            h: 1,
        };
        assert_eq!(smart_selection(click, &[monitor, window]), window);
        let outside = Rect {
            x: 1500,
            y: 900,
            w: 2,
            h: 2,
        };
        assert_eq!(smart_selection(outside, &[monitor, window]), monitor);
        let drag = Rect {
            x: 5,
            y: 5,
            w: 300,
            h: 200,
        };
        assert_eq!(smart_selection(drag, &[monitor, window]), drag);
    }
    #[test]
    fn timestamps_are_civil_dates() {
        assert_eq!(timestamp(0), "1970-01-01_00-00-00");
        assert_eq!(timestamp(1_791_158_400), "2026-10-05_00-00-00");
        assert_eq!(timestamp(951_782_400 + 3661), "2000-02-29_01-01-01");
    }
}
