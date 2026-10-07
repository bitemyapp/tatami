// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Tatami menu in Walker. Its tree is Elephant menu definitions in
//! /etc/xdg/tatami/elephant/menus, after Omarchy 4's omarchy-menu.jsonc
//! without its Arch package, update and theme entries. This opens it at the
//! root or a submenu, opens the application list, and shows the pickers whose
//! entries are only known at run time.
use crate::{hypr, util};
use std::{collections::HashMap, thread, time::Duration};

/// Submenus that can be opened directly, with the title Omarchy gives them.
pub const MENUS: &[(&str, &str)] = &[
    ("learn", "Learn"),
    ("trigger", "Trigger"),
    ("capture", "Capture"),
    ("screenrecord", "Screenrecord"),
    ("toggle", "Toggle"),
    ("hardware", "Hardware"),
    ("setup", "Setup"),
    ("display", "Display"),
    ("network", "Network"),
    ("dns", "DNS"),
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

/// The size of the theme's card. Walker keeps the last size it was given and
/// applies one only when asked, so every view states its size: this one
/// unless the caller's own arguments, which come later and win, say
/// otherwise. Without it, the menu opened after the keybindings or the
/// background picker kept their width.
const SIZE: [&str; 10] = [
    "--width",
    "300",
    "--minwidth",
    "1",
    "--maxwidth",
    "260",
    "--minheight",
    "1",
    "--maxheight",
    "636",
];

fn sized<'a>(args: &[&'a str]) -> Vec<&'a str> {
    SIZE.iter().copied().chain(args.iter().copied()).collect()
}

/// Open a Walker view.
pub fn walker(args: &[&str]) {
    util::spawn("walker", &sized(args));
}

/// A Walker list or prompt: what was chosen or typed.
pub fn walker_filter(args: &[&str], input: &[u8]) -> util::Result<String> {
    util::filter("walker", &sized(args), input)
}

/// Super+Space, Super+Escape and the bar's menu button. Walker closes
/// itself when asked to open while open, so the same key toggles the menu.
pub fn show(name: &str) {
    if let Some(args) = walker_args(name) {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        walker(&args);
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
    walker(&["-m", "desktopapplications", "-p", "Apps…"]);
}

/// Trigger › Emoji and Super+Ctrl+E.
pub fn emoji() {
    after_menu();
    walker(&["-m", "symbols", "-p", "Emojis…"]);
}

/// Super+Ctrl+V: the clipboard history.
pub fn clipboard() {
    after_menu();
    walker(&["-m", "clipboard", "-p", "Clipboard…"]);
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

/// The Walker theme for the bar's panels: dropped from the top right, under
/// the bar, like Omarchy 4's panels, instead of centered like the menu.
pub const PANEL_THEME: &str = "tatami-panel";
const PANEL_WIDTH: &str = "380";

/// A panel from the bar: a Walker list by position, for lists whose labels
/// may repeat, with the current entry selected.
pub fn panel_index(prompt: &str, lines: &[String], current: Option<usize>) -> Option<usize> {
    let input = lines.join("\n");
    let placeholder = format!("{prompt}\u{2026}");
    let selected = current.map(|index| (index + 1).to_string());
    let mut args = vec![
        "--dmenu",
        "--index",
        "--theme",
        PANEL_THEME,
        "--width",
        PANEL_WIDTH,
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
    let chosen = walker_filter(&args, input.as_bytes()).ok()?;
    chosen
        .trim()
        .parse()
        .ok()
        .filter(|index| *index < lines.len())
}

/// The bar's display icon and Super+Ctrl+D: Setup › Monitors as a panel.
pub fn display() {
    after_menu();
    walker(&[
        "--theme",
        PANEL_THEME,
        "-m",
        "menus:tatami-display",
        "-p",
        "Display\u{2026}",
    ]);
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
    if let Some(index) = panel_index("Power Profile", &lines, current) {
        util::run("powerprofilesctl", &["set", &lines[index]]);
    }
}

/// Modifiers in the order Omarchy's keybinding list names them.
const MODIFIERS: [(u64, &str); 6] = [
    (64, "SUPER"),
    (1, "SHIFT"),
    (4, "CTRL"),
    (8, "ALT"),
    (32, "MOD3"),
    (128, "MOD5"),
];

/// Readable key combination from `hyprctl binds -j` fields, written as
/// Omarchy writes it: "SUPER SHIFT + RETURN".
pub fn combination(modmask: u64, key: &str) -> String {
    let modifiers: Vec<&str> = MODIFIERS
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
    let key = key.to_uppercase();
    if modifiers.is_empty() {
        key
    } else {
        format!("{} + {key}", modifiers.join(" "))
    }
}

/// Where a binding sorts in the list, after Omarchy's omarchy-menu-keybindings:
/// the everyday bindings first, media keys last. As there, the last rule that
/// matches wins.
fn priority(keys: &str, description: &str) -> u8 {
    let has = |text: &str| description.contains(text);
    let rules = [
        (has("Keybindings"), 0),
        (has("Tatami menu"), 1),
        (has("Terminal"), 2),
        (has("Browser") && !has("Browser ("), 3),
        (has("File manager") && !has("(cwd)"), 4),
        (has("Apps menu"), 5),
        (has("System menu"), 6),
        (has("Full screen"), 8),
        (has("Full width"), 9),
        (has("Close window"), 10),
        (has("Close all windows"), 11),
        (has("Lock system"), 12),
        (has("Toggle window floating"), 13),
        (has("Toggle window split"), 14),
        (has("Pop window"), 15),
        (has("Universal"), 16),
        (has("Clipboard"), 17),
        (has("Audio"), 18),
        (has("Bluetooth"), 19),
        (has("Wi-Fi"), 20),
        (has("Emojis"), 21),
        (has("Color picker"), 22),
        (has("Screenshot"), 23),
        (has("Screenrecording"), 24),
        (keys == "SUPER SHIFT + B" && description == "Browser", 27),
        (has("File manager (cwd)"), 28),
        (
            ["Switch", "Next", "Former", "Previous"]
                .iter()
                .any(|word| has(word))
                && has("workspace"),
            29,
        ),
        (has("Move window to workspace"), 30),
        (has("Move window silently to workspace"), 31),
        (has("Swap window"), 32),
        (has("Focus"), 33),
        (description.ends_with("Move window"), 34),
        (has("Resize window"), 35),
        (has("Expand window"), 36),
        (has("Shrink window"), 37),
        (has("scratchpad"), 38),
        (has("notification"), 39),
        (has("Toggle window transparency"), 40),
        (has("Toggle workspace gaps"), 41),
        (has("Toggle nightlight"), 42),
        (has("Toggle locking"), 43),
        (has("group"), 94),
        (has("Scroll active workspace"), 95),
        (has("Reveal active"), 97),
        (keys.contains("COPILOT"), 50),
        (keys.contains("XF86"), 99),
    ];
    rules
        .iter()
        .filter(|(matches, _)| *matches)
        .map(|(_, priority)| *priority)
        .next_back()
        .unwrap_or(50)
}

/// Keys of bindings by keycode ("SUPER + code:10"), which Hyprland reports
/// for Lua configurations with neither key nor code: by modifiers and
/// description, from the configuration's `hl.bind("…", …, { description =
/// "…" })` lines, as omarchy-menu-keybindings reads them from its source.
pub fn source_keys(config: &str) -> HashMap<(u64, String), String> {
    let quoted = |text: &str| -> Option<String> {
        let start = text.find('"')? + 1;
        let end = start + text[start..].find('"')?;
        Some(text[start..end].to_owned())
    };
    config
        .lines()
        .filter_map(|line| {
            let call = line.trim_start().strip_prefix("hl.bind(")?;
            let combination = quoted(call)?;
            let description = quoted(call.split_once("description =")?.1)?;
            let mut modmask = 0;
            let mut key = String::new();
            for part in combination.split('+').map(str::trim) {
                match MODIFIERS
                    .iter()
                    .find(|(_, name)| part.eq_ignore_ascii_case(name))
                    .or_else(|| {
                        part.eq_ignore_ascii_case("CONTROL")
                            .then_some(&MODIFIERS[2])
                    }) {
                    Some((bit, _)) => modmask |= bit,
                    None => key = part.to_owned(),
                }
            }
            Some(((modmask, description), key))
        })
        .collect()
}

/// "SUPER + K                           → Keybindings" rows, in Omarchy's order.
pub fn keybinding_lines(
    binds: &serde_json::Value,
    keys: &HashMap<(u64, String), String>,
) -> Vec<String> {
    let mut rows: Vec<(u8, String)> = binds
        .as_array()
        .into_iter()
        .flatten()
        .filter(|bind| bind["has_description"] == true)
        .filter_map(|bind| {
            let description = bind["description"].as_str()?;
            let modmask = bind["modmask"].as_u64().unwrap_or(0);
            let key = match bind["key"].as_str()? {
                "" => keys.get(&(modmask, description.to_owned()))?.as_str(),
                key => key,
            };
            let combination = combination(modmask, key);
            Some((
                priority(&combination, description),
                format!("{combination:<35} \u{2192} {description}"),
            ))
        })
        .collect();
    rows.sort();
    rows.dedup();
    rows.into_iter().map(|(_, line)| line).collect()
}

/// Super+K and Learn › Keybindings: every described binding.
pub fn keybindings() {
    after_menu();
    // The shipped configuration, then the user's, which loads it and adds
    // its own bindings (Setup › Hyprland).
    let config = [
        format!("{}/hypr/hyprland.lua", crate::CONFIG),
        format!("{}/.config/tatami/hyprland.lua", util::home()),
    ]
    .iter()
    .map(|path| std::fs::read_to_string(path).unwrap_or_default())
    .collect::<Vec<_>>()
    .join("\n");
    let lines = hypr::json("binds")
        .map(|binds| keybinding_lines(&binds, &source_keys(&config)))
        .unwrap_or_default();
    let input = lines.join("\n");
    let _ = walker_filter(
        &[
            "--dmenu",
            "--width",
            "800",
            "--maxheight",
            "450",
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
        let lines = keybinding_lines(&binds, &HashMap::new());
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("SUPER + W ") && lines[0].ends_with("\u{2192} Close window"));
        assert!(lines[1].starts_with("SUPER SHIFT + 1 "));
    }
    #[test]
    fn keys_of_keycode_bindings_come_from_the_configuration() {
        let config = r#"
-- comment hl.bind("SUPER + X", nothing)
hl.bind("SUPER + code:10", hl.dsp.focus({ workspace = "1" }), { description = "Switch to workspace 1" })
  hl.bind("SUPER + CTRL + SHIFT + code:21", hl.dsp.window.resize({ x = 0, y = 300, relative = true }), { description = "Expand window down a lot" })
hl.bind("SUPER + K", hl.dsp.exec_cmd("tatami keybindings"))
"#;
        let keys = source_keys(config);
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[&(64, "Switch to workspace 1".to_owned())], "code:10");
        assert_eq!(
            keys[&(69, "Expand window down a lot".to_owned())],
            "code:21"
        );
    }
    #[test]
    fn keybinding_help_puts_everyday_bindings_first_and_media_keys_last() {
        let binds = serde_json::json!([
            {"modmask": 0, "key": "XF86AudioMute", "has_description": true, "description": "Mute"},
            {"modmask": 64, "key": "left", "has_description": true, "description": "Focus on left window"},
            {"modmask": 64, "key": "Return", "has_description": true, "description": "Terminal"},
            {"modmask": 65, "key": "B", "has_description": true, "description": "Browser"},
            {"modmask": 65, "key": "Return", "has_description": true, "description": "Browser"},
            {"modmask": 64, "key": "K", "has_description": true, "description": "Keybindings"},
            {"modmask": 65, "key": "", "keycode": 0, "has_description": true, "description": "Tatami menu"},
            {"modmask": 64, "key": "", "keycode": 0, "has_description": true, "description": "Unknown"}
        ]);
        let config = r#"hl.bind("SUPER + SHIFT + code:201", hl.dsp.exec_cmd("tatami menu"), { description = "Tatami menu" })"#;
        let order: Vec<String> = keybinding_lines(&binds, &source_keys(config))
            .iter()
            .map(|line| line.split('\u{2192}').next().unwrap().trim().to_owned())
            .collect();
        assert_eq!(
            order,
            [
                "SUPER + K",
                "SUPER + RETURN",
                "SUPER SHIFT + RETURN",
                "SUPER SHIFT + B",
                "SUPER + LEFT",
                "SUPER SHIFT + COPILOT",
                "XF86AUDIOMUTE"
            ]
        );
    }
}
