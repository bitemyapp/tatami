// SPDX-License-Identifier: GPL-3.0-or-later
//! Locking, waking and leaving the session.
use crate::{CONFIG, hypr, util};
use std::{thread, time::Duration};

fn locked() -> bool {
    !util::pids_named(&["hyprlock", ".hyprlock-wrapped"]).is_empty()
}

/// Lock the session, then turn the displays off after a few seconds unless
/// `lock_only` (before suspend the displays turn off anyway).
pub fn lock(lock_only: bool) {
    if !locked()
        && let Ok(me) = std::env::current_exe()
        && let Some(me) = me.to_str()
    {
        // Wait for hyprlock in a separate process so this command returns
        // promptly to hypridle and key bindings.
        util::spawn(me, &["lock-wait"]);
    }
    // Unlock with the primary layout, whatever was active when locking.
    util::run("hyprctl", &["switchxkblayout", "all", "0"]);
    if !lock_only {
        thread::sleep(Duration::from_secs(3));
        if locked() {
            hypr::dpms(false);
        }
    }
}

pub fn lock_wait() {
    util::run("hyprlock", &["-c", &format!("{CONFIG}/hypr/hyprlock.conf")]);
    wake(0);
}

pub fn wake(delay: u64) {
    thread::sleep(Duration::from_secs(delay));
    hypr::dpms(true);
}

/// Close windows politely so applications can save state, then focus the
/// first workspace.
pub fn close_all() {
    if let Ok(clients) = hypr::json("clients") {
        for address in hypr::window_addresses(&clients) {
            hypr::dispatch(&format!(
                "hl.dsp.window.close({{ window = \"address:{address}\" }})"
            ));
        }
    }
    hypr::dispatch("hl.dsp.focus({ workspace = 1 })");
}

pub enum Leave {
    Logout,
    Reboot,
    Shutdown,
}

pub fn leave(how: Leave) {
    close_all();
    // Give browsers a moment to shut down cleanly.
    thread::sleep(Duration::from_secs(1));
    match how {
        Leave::Logout => util::run("uwsm", &["stop"]),
        Leave::Reboot => util::run("systemctl", &["reboot", "--no-wall"]),
        Leave::Shutdown => util::run("systemctl", &["poweroff", "--no-wall"]),
    };
}

/// Hibernation needs active swap and kernel support for suspend-to-disk.
pub fn can_hibernate(proc_swaps: &str, power_state: &str) -> bool {
    proc_swaps.lines().count() > 1 && power_state.split_whitespace().any(|s| s == "disk")
}

pub fn hibernation_available() -> bool {
    can_hibernate(
        &std::fs::read_to_string("/proc/swaps").unwrap_or_default(),
        &std::fs::read_to_string("/sys/power/state").unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hibernation_requires_swap_and_disk_state() {
        let swaps = "Filename Type Size Used Priority\n/dev/nvme0n1p3 partition 33554428 0 -2\n";
        assert!(can_hibernate(swaps, "freeze mem disk\n"));
        assert!(!can_hibernate(
            "Filename Type Size Used Priority\n",
            "freeze mem disk"
        ));
        assert!(!can_hibernate(swaps, "freeze mem"));
    }
}
