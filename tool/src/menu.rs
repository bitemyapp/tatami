// SPDX-License-Identifier: GPL-3.0-or-later
//! Walker dmenu menus, adapted from Omarchy's menu without its Arch package,
//! update and theme-installation entries.
use crate::{power, util};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Menu(&'static str),
    /// Run this helper with arguments.
    Helper(&'static [&'static str]),
    /// Run another program.
    Program(&'static [&'static str]),
    Launcher,
}

pub struct Item {
    pub icon: &'static str,
    pub label: &'static str,
    pub action: Action,
}
impl Item {
    pub fn line(&self) -> String {
        format!("{}  {}", self.icon, self.label)
    }
}

const fn item(icon: &'static str, label: &'static str, action: Action) -> Item {
    Item {
        icon,
        label,
        action,
    }
}

/// Prompt and entries of a named menu. `hibernate` hides Hibernate when the
/// machine has no swap to resume from.
pub fn menu(name: &str, hibernate: bool) -> Option<(&'static str, Vec<Item>)> {
    use Action::*;
    Some(match name {
        "main" => (
            "Go",
            vec![
                item("\u{f003b}", "Apps", Launcher),
                item("\u{f030}", "Capture", Menu("capture")),
                item("\u{f050e}", "Toggle", Menu("toggle")),
                item("\u{e615}", "Setup", Menu("setup")),
                item("\u{f11c}", "Keybindings", Helper(&["keybindings"])),
                item("\u{ea74}", "About", Helper(&["about"])),
                item("\u{f011}", "System", Menu("system")),
            ],
        ),
        "system" => {
            let mut items = vec![
                item("\u{f023}", "Lock", Helper(&["lock"])),
                item("\u{f04b2}", "Suspend", Program(&["systemctl", "suspend"])),
            ];
            if hibernate {
                items.push(item(
                    "\u{f0901}",
                    "Hibernate",
                    Program(&["systemctl", "hibernate"]),
                ));
            }
            items.extend([
                item("\u{f0343}", "Logout", Helper(&["logout"])),
                item("\u{f0709}", "Restart", Helper(&["reboot"])),
                item("\u{f0425}", "Shutdown", Helper(&["shutdown"])),
            ]);
            ("System", items)
        }
        "capture" => (
            "Capture",
            vec![
                item("\u{f030}", "Screenshot", Helper(&["screenshot", "smart"])),
                item(
                    "\u{f030}",
                    "Screenshot region",
                    Helper(&["screenshot", "region"]),
                ),
                item(
                    "\u{f030}",
                    "Screenshot display",
                    Helper(&["screenshot", "fullscreen"]),
                ),
                item("\u{f00c9}", "Color", Helper(&["color-picker"])),
            ],
        ),
        "toggle" => (
            "Toggle",
            vec![
                item("\u{f050e}", "Nightlight", Helper(&["toggle", "nightlight"])),
                item("\u{f1ad6}", "Idle Lock", Helper(&["toggle", "idle"])),
                item(
                    "\u{f009b}",
                    "Notifications",
                    Helper(&["toggle", "notifications"]),
                ),
                item("\u{f035c}", "Top Bar", Helper(&["toggle", "bar"])),
            ],
        ),
        "setup" => (
            "Setup",
            vec![
                item("\u{e638}", "Audio", Helper(&["tui", "wiremix"])),
                item("\u{f1eb}", "Wifi", Helper(&["tui", "nmtui"])),
                item("\u{f00af}", "Bluetooth", Helper(&["tui", "bluetui"])),
                item("\u{f140b}", "Power Profile", Menu("power")),
            ],
        ),
        _ => return None,
    })
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

fn pick(prompt: &str, lines: &[String], current: Option<usize>) -> Option<String> {
    let input = lines.join("\n");
    let placeholder = format!("{prompt}\u{2026}");
    let selected = current.map(|index| (index + 1).to_string());
    let mut args = vec![
        "--dmenu",
        "--width",
        "295",
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

fn run_action(action: Action) {
    match action {
        Action::Helper(args) => {
            if let Ok(me) = std::env::current_exe()
                && let Some(me) = me.to_str()
            {
                util::spawn(me, args);
            }
        }
        Action::Program(argv) => util::spawn(argv[0], &argv[1..]),
        Action::Launcher => util::spawn("walker", &["-p", "Launch\u{2026}"]),
        Action::Menu(_) => {}
    }
}

/// Show a menu. Escape returns to the main menu, or exits when the menu
/// was opened directly (e.g. Super+Escape for System).
pub fn show(start: &str) {
    let direct = start != "main";
    let mut name = start.to_owned();
    let hibernate = power::hibernation_available();
    loop {
        if name == "power" {
            let list = util::output("powerprofilesctl", &["list"]).unwrap_or_default();
            let profiles = power_profiles(&list);
            let lines: Vec<String> = profiles.iter().map(|(name, _)| name.clone()).collect();
            let current = profiles.iter().position(|(_, current)| *current);
            if let Some(profile) = pick("Power Profile", &lines, current) {
                util::run("powerprofilesctl", &["set", &profile]);
            }
            return;
        }
        let Some((prompt, items)) = menu(&name, hibernate) else {
            return;
        };
        let lines: Vec<String> = items.iter().map(Item::line).collect();
        let Some(chosen) = pick(prompt, &lines, None) else {
            if direct || name == "main" {
                return;
            }
            name = "main".into();
            continue;
        };
        let Some(item) = items.iter().find(|item| item.line() == chosen) else {
            return;
        };
        match item.action {
            Action::Menu(next) => name = next.into(),
            action => return run_action(action),
        }
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
        "mouse:272" => "LEFT CLICK",
        "mouse:273" => "RIGHT CLICK",
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
            Some(format!("{keys:<28} {description}"))
        })
        .collect();
    lines.dedup();
    lines
}

pub fn keybindings() {
    let lines = crate::hypr::json("binds")
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
    fn every_submenu_exists_and_labels_are_unique() {
        for name in ["main", "system", "capture", "toggle", "setup"] {
            for hibernate in [true, false] {
                let (_, items) = menu(name, hibernate).unwrap();
                let mut lines: Vec<_> = items.iter().map(Item::line).collect();
                let count = lines.len();
                lines.sort();
                lines.dedup();
                assert_eq!(lines.len(), count, "{name}");
                for item in items {
                    if let Action::Menu(next) = item.action {
                        assert!(next == "power" || menu(next, hibernate).is_some());
                    }
                }
            }
        }
        let has = |hibernate| {
            menu("system", hibernate)
                .unwrap()
                .1
                .iter()
                .any(|item| item.label == "Hibernate")
        };
        assert!(has(true) && !has(false));
        assert!(menu("install", true).is_none());
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
