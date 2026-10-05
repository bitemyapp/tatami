// SPDX-License-Identifier: GPL-3.0-or-later
//! Session toggles and the Waybar indicators that reflect them.
use crate::util;
use std::{thread, time::Duration};

const IDLE_UNIT: &str = "omarchy-hypridle.service";
const BAR_UNIT: &str = "omarchy-waybar.service";
const SUNSET_UNIT: &str = "omarchy-hyprsunset.service";
pub const IDLE_SIGNAL: i32 = 9;
pub const SILENCE_SIGNAL: i32 = 10;
const NIGHT: u32 = 4000;
const DAY: u32 = 6000;

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

pub fn idle() {
    if flip(IDLE_UNIT) {
        util::notify("\u{f1ad6}    Now locking computer when idle", "");
    } else {
        util::notify("\u{f1ad6}    Stop locking computer when idle", "");
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
        util::notify("\u{f009b}    Silenced notifications", "");
    } else {
        util::notify("\u{f009a}    Enabled notifications", "");
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

pub fn nightlight() {
    if !active(SUNSET_UNIT) {
        util::run("systemctl", &["--user", "start", SUNSET_UNIT]);
        thread::sleep(Duration::from_secs(1));
    }
    let current = util::output("hyprctl", &["hyprsunset", "temperature"])
        .ok()
        .and_then(|text| parse_temperature(&text));
    let night = current != Some(NIGHT);
    let target = if night { NIGHT } else { DAY };
    util::run(
        "hyprctl",
        &["hyprsunset", "temperature", &target.to_string()],
    );
    if night {
        util::notify("\u{f0594}    Nightlight screen temperature", "");
    } else {
        util::notify("\u{f0599}    Daylight screen temperature", "");
    }
}

/// Waybar custom module JSON: empty when the normal state holds.
pub fn indicator(which: &str) -> String {
    let (text, tooltip) = match which {
        "idle" if !active(IDLE_UNIT) => ("\u{f1ad6}", "Idle lock disabled"),
        "notifications" if silenced() => ("\u{f009b}", "Notifications silenced"),
        _ => return serde_json::json!({ "text": "" }).to_string(),
    };
    serde_json::json!({ "text": text, "tooltip": tooltip, "class": "active" }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn temperature_is_the_first_number() {
        assert_eq!(parse_temperature("6000\n"), Some(6000));
        assert_eq!(parse_temperature("temperature: 4000K"), Some(4000));
        assert_eq!(parse_temperature("couldn't connect"), None);
    }
}
