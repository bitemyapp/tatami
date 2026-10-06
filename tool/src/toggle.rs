// SPDX-License-Identifier: MIT OR Apache-2.0
//! Session toggles and the bar indicators that reflect them, after
//! Omarchy 4's Stay Awake, Do Not Disturb and Night Light indicators.
use crate::util;
use std::{thread, time::Duration};

const IDLE_UNIT: &str = "tatami-hypridle.service";
const BAR_UNIT: &str = "tatami-waybar.service";
const SUNSET_UNIT: &str = "tatami-hyprsunset.service";
pub const IDLE_SIGNAL: i32 = 9;
pub const SILENCE_SIGNAL: i32 = 10;
pub const NIGHTLIGHT_SIGNAL: i32 = 11;
const NIGHT: u32 = 4000;
const DAY: u32 = 6500;
/// Below this the screen is tinted (hyprsunset's identity is 6000 K).
const IDENTITY: u32 = 6000;

fn active(unit: &str) -> bool {
    util::run("systemctl", &["--user", "--quiet", "is-active", unit])
}

fn flip(unit: &str) -> bool {
    let now = !active(unit);
    util::run(
        "systemctl",
        &["--user", if now { "start" } else { "stop" }, unit],
    );
    now
}

/// Stay Awake: stop or start locking and screen blanking on idle.
pub fn idle() {
    if flip(IDLE_UNIT) {
        util::notify("\u{f0176}", "Locking the computer when idle", "");
    } else {
        util::notify("\u{f0176}", "Staying awake", "");
    }
    util::signal_waybar(IDLE_SIGNAL);
}

pub fn bar() {
    flip(BAR_UNIT);
}

fn silenced() -> bool {
    util::output("makoctl", &["mode"]).is_ok_and(|modes| modes.contains("do-not-disturb"))
}

pub fn notifications() {
    util::run("makoctl", &["mode", "-t", "do-not-disturb"]);
    if silenced() {
        // Visible by design: the do-not-disturb mode still shows notify-send.
        util::notify("\u{f009b}", "Silenced notifications", "");
    } else {
        util::notify("\u{f009a}", "Enabled notifications", "");
    }
    util::signal_waybar(SILENCE_SIGNAL);
}

/// The temperature from `hyprctl hyprsunset temperature`.
pub fn parse_temperature(text: &str) -> Option<u32> {
    text.split(|c: char| !c.is_ascii_digit())
        .find(|word| !word.is_empty())?
        .parse()
        .ok()
}

fn temperature() -> Option<u32> {
    util::output("hyprctl", &["hyprsunset", "temperature"])
        .ok()
        .and_then(|text| parse_temperature(&text))
}

/// The temperature to switch to: night unless the screen is already tinted.
pub fn next_temperature(current: Option<u32>) -> u32 {
    match current {
        Some(t) if t < IDENTITY => DAY,
        _ => NIGHT,
    }
}

pub fn nightlight() {
    if !active(SUNSET_UNIT) {
        util::run("systemctl", &["--user", "start", SUNSET_UNIT]);
    }
    // hyprsunset accepts requests only once its socket is up.
    let mut current = None;
    for _ in 0..10 {
        current = temperature();
        if current.is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }
    let target = next_temperature(current);
    for _ in 0..10 {
        util::run(
            "hyprctl",
            &["hyprsunset", "temperature", &target.to_string()],
        );
        if temperature() == Some(target) {
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }
    if target == NIGHT {
        util::notify("\u{f0594}", "Nightlight screen temperature", "");
    } else {
        util::notify("\u{f0599}", "Daylight screen temperature", "");
    }
    util::signal_waybar(NIGHTLIGHT_SIGNAL);
}

/// Waybar custom module JSON. The glyph is always there so the slot keeps
/// its place beside the clock; the "inactive" class hides it until hovered.
pub fn indicator_json(glyph: &str, active: bool, on: &str, off: &str) -> String {
    serde_json::json!({
        "text": glyph,
        "tooltip": if active { on } else { off },
        "class": if active { "active" } else { "inactive" },
    })
    .to_string()
}

pub fn indicator(which: &str) -> String {
    match which {
        "nightlight" => indicator_json(
            "\u{f050e}",
            active(SUNSET_UNIT) && temperature().is_some_and(|t| t < IDENTITY),
            "Day Light",
            "Night Light",
        ),
        "notifications" => indicator_json(
            "\u{f009b}",
            silenced(),
            "Allow Notifications",
            "Silence Notifications",
        ),
        "idle" => indicator_json(
            "\u{f0176}",
            !active(IDLE_UNIT),
            "Allow Idle Lock & Screensaver",
            "Stay Awake",
        ),
        "screenrecording" => indicator_json(
            "\u{f0ec2}",
            crate::capture::recording(),
            "Stop recording",
            "Screen Recording",
        ),
        "tray" => tray_indicator(tray_items()),
        _ => serde_json::json!({ "text": "" }).to_string(),
    }
}

/// Applications with a tray icon, from the bar's StatusNotifierWatcher.
fn tray_items() -> usize {
    util::output(
        "busctl",
        &[
            "--user",
            "--json=short",
            "get-property",
            "org.kde.StatusNotifierWatcher",
            "/StatusNotifierWatcher",
            "org.kde.StatusNotifierWatcher",
            "RegisteredStatusNotifierItems",
        ],
    )
    .ok()
    .and_then(|reply| serde_json::from_str::<serde_json::Value>(&reply).ok())
    .and_then(|reply| reply["data"].as_array().map(Vec::len))
    .unwrap_or(0)
}

/// The tray's opener: a chevron while there are tray icons, otherwise
/// nothing, which Waybar hides.
pub fn tray_indicator(items: usize) -> String {
    let text = if items > 0 { "\u{f053}" } else { "" };
    serde_json::json!({ "text": text }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tray_opener_only_with_tray_icons() {
        assert_eq!(tray_indicator(0), r#"{"text":""}"#);
        assert!(tray_indicator(2).contains('\u{f053}'));
    }
    #[test]
    fn temperature_is_the_first_number() {
        assert_eq!(parse_temperature("6000\n"), Some(6000));
        assert_eq!(parse_temperature("temperature: 4000K"), Some(4000));
        assert_eq!(parse_temperature("couldn't connect"), None);
    }
    #[test]
    fn nightlight_alternates_like_tatami() {
        assert_eq!(next_temperature(None), 4000);
        assert_eq!(next_temperature(Some(6000)), 4000);
        assert_eq!(next_temperature(Some(6500)), 4000);
        assert_eq!(next_temperature(Some(4000)), 6500);
    }
    #[test]
    fn indicators_keep_their_slot() {
        let off: serde_json::Value =
            serde_json::from_str(&indicator_json("x", false, "on", "off")).unwrap();
        assert_eq!(off["text"], "x");
        assert_eq!(off["class"], "inactive");
        assert_eq!(off["tooltip"], "off");
        let on: serde_json::Value =
            serde_json::from_str(&indicator_json("x", true, "on", "off")).unwrap();
        assert_eq!(on["class"], "active");
    }
}
