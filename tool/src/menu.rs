// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Tatami menu in Walker. Its tree is Elephant menu definitions in
//! /etc/xdg/tatami/elephant/menus, after Omarchy 4's omarchy-menu.jsonc
//! without its Arch package, update and theme entries. This opens it at the
//! root or a submenu, opens the application list, and shows the pickers whose
//! entries are only known at run time.
use crate::{hypr, util};
use std::{thread, time::Duration};

/// Submenus that can be opened directly, with the title Omarchy gives them.
pub const MENUS: &[(&str, &str)] = &[
    ("learn", "Learn"),
    ("trigger", "Trigger"),
    ("capture", "Capture"),
    ("toggle", "Toggle"),
    ("hardware", "Hardware"),
    ("setup", "Setup"),
    ("display", "Display"),
    ("system", "System"),
];

/// Walker arguments for a menu. The root uses the "tatami" provider set,
/// whose search also finds applications, as Omarchy's root menu does.
pub fn walker_args(name: &str) -> Option<Vec<String>> {
    if matches!(name, "" | "root") {
        return Some(vec![
            "-s".into(),
            "tatami".into(),
            "-p".into(),
            "Go…".into(),
        ]);
    }
    let (_, title) = MENUS.iter().find(|(id, _)| *id == name)?;
    Some(vec![
        "-m".into(),
        format!("menus:tatami-{name}"),
        "-p".into(),
        format!("{title}…"),
    ])
}

/// Super+Space, Super+Escape and the bar's menu button. Walker closes
/// itself when asked to open while open, so the same key toggles the menu.
pub fn show(name: &str) {
    if let Some(args) = walker_args(name) {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        util::spawn("walker", &args);
    }
}

fn walker_open() -> bool {
    hypr::json("layers").is_ok_and(|layers| layers.to_string().contains("\"namespace\":\"walker\""))
}

/// A menu entry that opens another Walker view runs while the menu is
/// still closing; wait for it, or Walker would take the request as "close".
pub fn after_menu() {
    for _ in 0..40 {
        if !walker_open() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
}

/// The application list (Super+Alt+Space, and Apps in the menu).
pub fn apps() {
    after_menu();
    util::spawn("walker", &["-m", "desktopapplications", "-p", "Apps…"]);
}

/// Trigger › Emoji.
pub fn emoji() {
    after_menu();
    util::spawn("walker", &["-m", "symbols", "-p", "Emojis…"]);
}

/// Wait for the menu to close before capturing the screen.
pub fn before_capture() {
    after_menu();
}

/// Profiles from `powerprofilesctl list` (the active one is marked `*`).
pub fn power_profiles(list: &str) -> Vec<(String, bool)> {
    list.lines()
        .filter(|line| !line.starts_with("    ") && line.trim_end().ends_with(':'))
        .map(|line| {
            let current = line.trim_start().starts_with('*');
            let name = line
                .trim()
                .trim_start_matches('*')
                .trim()
                .trim_end_matches(':')
                .to_owned();
            (name, current)
        })
        .filter(|(name, _)| !name.is_empty())
        .collect()
}

fn pick(prompt: &str, lines: &[String], current: Option<usize>, width: &str) -> Option<String> {
    let input = lines.join("\n");
    let placeholder = format!("{prompt}\u{2026}");
    let selected = current.map(|index| (index + 1).to_string());
    let mut args = vec![
        "--dmenu",
        "--width",
        width,
        "--minheight",
        "1",
        "--maxheight",
        "630",
        "-p",
        placeholder.as_str(),
    ];
    if let Some(selected) = &selected {
        args.extend(["-c", selected.as_str()]);
    }
    let chosen = util::filter("walker", &args, input.as_bytes()).ok()?;
    let chosen = chosen.trim();
    lines.iter().find(|line| line.as_str() == chosen).cloned()
}

/// Like `pick`, by position, for lists whose labels may repeat.
pub fn pick_index(
    prompt: &str,
    lines: &[String],
    current: Option<usize>,
    width: &str,
) -> Option<usize> {
    let input = lines.join("\n");
    let placeholder = format!("{prompt}\u{2026}");
    let selected = current.map(|index| (index + 1).to_string());
    let mut args = vec![
        "--dmenu",
        "--index",
        "--width",
        width,
        "--minheight",
        "1",
        "--maxheight",
        "630",
        "-p",
        placeholder.as_str(),
    ];
    if let Some(selected) = &selected {
        args.extend(["-c", selected.as_str()]);
    }
    let chosen = util::filter("walker", &args, input.as_bytes()).ok()?;
    chosen
        .trim()
        .parse()
        .ok()
        .filter(|index| *index < lines.len())
}

/// Setup › Power Profile and the bar's battery: the profiles this machine
/// offers, with the active one selected.
pub fn power_profile() {
    after_menu();
    let list = util::output("powerprofilesctl", &["list"]).unwrap_or_default();
    let profiles = power_profiles(&list);
    if profiles.is_empty() {
        util::notify("\u{f140b}", "Power profiles are not available", "");
        return;
    }
    let lines: Vec<String> = profiles.iter().map(|(name, _)| name.clone()).collect();
    let current = profiles.iter().position(|(_, current)| *current);
    if let Some(profile) = pick("Power Profile", &lines, current, "300") {
        util::run("powerprofilesctl", &["set", &profile]);
    }
}

const MODIFIERS: [(u64, &str); 6] = [
    (64, "SUPER"),
    (4, "CTRL"),
    (8, "ALT"),
    (1, "SHIFT"),
    (32, "MOD3"),
    (128, "MOD5"),
];

/// Readable key combination from `hyprctl binds -j` fields.
pub fn combination(modmask: u64, key: &str) -> String {
    let mut parts: Vec<&str> = MODIFIERS
        .iter()
        .filter(|(bit, _)| modmask & bit != 0)
        .map(|(_, name)| *name)
        .collect();
    let key = match key {
        "code:10" => "1",
        "code:11" => "2",
        "code:12" => "3",
        "code:13" => "4",
        "code:14" => "5",
        "code:15" => "6",
        "code:16" => "7",
        "code:17" => "8",
        "code:18" => "9",
        "code:19" => "0",
        "code:20" => "-",
        "code:21" => "=",
        "code:34" => "[",
        "code:35" => "]",
        "code:201" => "COPILOT",
        "mouse:272" => "LEFT MOUSE BUTTON",
        "mouse:273" => "RIGHT MOUSE BUTTON",
        "mouse_down" => "SCROLL DOWN",
        "mouse_up" => "SCROLL UP",
        other => other,
    };
    let upper = key.to_uppercase();
    parts.push(&upper);
    parts.join(" + ")
}

pub fn keybinding_lines(binds: &serde_json::Value) -> Vec<String> {
    let mut lines: Vec<String> = binds
        .as_array()
        .into_iter()
        .flatten()
        .filter(|bind| bind["has_description"] == true)
        .filter_map(|bind| {
            let description = bind["description"].as_str()?;
            let key = bind["key"].as_str()?;
            let keys = combination(bind["modmask"].as_u64().unwrap_or(0), key);
            Some(format!("{keys:<32} {description}"))
        })
        .collect();
    lines.dedup();
    lines
}

/// Super+K and Learn › Keybindings: every described binding.
pub fn keybindings() {
    after_menu();
    let lines = hypr::json("binds")
        .map(|binds| keybinding_lines(&binds))
        .unwrap_or_default();
    let input = lines.join("\n");
    let _ = util::filter(
        "walker",
        &[
            "--dmenu",
            "--width",
            "800",
            "--maxheight",
            "630",
            "-p",
            "Keybindings\u{2026}",
        ],
        input.as_bytes(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn menus_open_at_the_root_or_a_submenu() {
        assert_eq!(walker_args("").unwrap()[..2], ["-s", "tatami"]);
        assert_eq!(
            walker_args("system").unwrap(),
            ["-m", "menus:tatami-system", "-p", "System…"]
        );
        assert!(walker_args("install").is_none());
    }
    #[test]
    fn power_profiles_parse_with_current_marker() {
        let list = "  performance:\n    CpuDriver:\tamd_pstate\n    Degraded:   no\n\n* balanced:\n    CpuDriver:\tamd_pstate\n\n  power-saver:\n    CpuDriver:\tamd_pstate\n";
        assert_eq!(
            power_profiles(list),
            [
                ("performance".to_owned(), false),
                ("balanced".to_owned(), true),
                ("power-saver".to_owned(), false)
            ]
        );
        assert!(power_profiles("").is_empty());
    }
    #[test]
    fn keybinding_help_lists_described_binds() {
        let binds = serde_json::json!([
            {"modmask": 64, "key": "W", "has_description": true, "description": "Close window"},
            {"modmask": 65, "key": "code:10", "has_description": true, "description": "Move window to workspace 1"},
            {"modmask": 0, "key": "XF86AudioMute", "has_description": false, "description": ""}
        ]);
        let lines = keybinding_lines(&binds);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("SUPER + W ") && lines[0].ends_with("Close window"));
        assert!(lines[1].starts_with("SUPER + SHIFT + 1 "));
    }
}
