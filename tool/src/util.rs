// SPDX-License-Identifier: MIT OR Apache-2.0
//! Small process helpers. Commands are always given as argument vectors; no
//! shell is involved, so selections and window titles are never interpreted.
use std::{
    io::Write,
    os::unix::process::CommandExt,
    process::{Command, Stdio},
};

pub type Result<T> = std::result::Result<T, String>;

/// Run a program and return its stdout, or an error naming the program.
pub fn output(program: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|error| format!("{program}: {error}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(format!("{program} exited with {}", out.status))
    }
}

/// Run a program to completion, ignoring its output.
pub fn run(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Start a program in its own session so it outlives this helper.
pub fn spawn(program: &str, args: &[&str]) {
    let _ = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn();
}

/// Start an application in its own systemd scope in app-graphical.slice, as
/// Omarchy does with uwsm-app. Otherwise it would run inside the
/// compositor's unit, where systemd-oomd under memory pressure would stop the
/// whole session instead of the one application. A scope runs the program
/// from here, so it keeps this session's environment (XDG_CONFIG_DIRS with
/// the Tatami configuration), which uwsm-app's daemon would not pass on.
pub fn launch(program: &str, args: &[&str]) {
    if which("systemd-run").is_some() {
        let mut full = vec![
            "--user",
            "--scope",
            "--quiet",
            "--collect",
            "--slice=app-graphical.slice",
            "--",
            program,
        ];
        full.extend_from_slice(args);
        spawn("systemd-run", &full);
    } else {
        spawn(program, args);
    }
}

/// This helper, for commands that run it again.
pub fn me() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.to_str().map(str::to_owned))
}

/// Feed `input` to a program's stdin and return its stdout.
pub fn filter(program: &str, args: &[&str], input: &[u8]) -> Result<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("{program}: {error}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        // A picker may exit before reading everything; that is not an error.
        let _ = stdin.write_all(input);
    }
    let out = child
        .wait_with_output()
        .map_err(|error| format!("{program}: {error}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// A low-urgency notification. Omarchy draws the glyph in the icon slot;
/// Mako has none, so the glyph leads the summary.
pub fn notify(glyph: &str, summary: &str, body: &str) {
    let summary = if glyph.is_empty() {
        summary.to_owned()
    } else {
        format!("{glyph}    {summary}")
    };
    let mut args = vec!["-u", "low", summary.as_str()];
    if !body.is_empty() {
        args.push(body);
    }
    run("notify-send", &args);
}

/// The first executable named `name` on PATH.
pub fn which(name: &str) -> Option<std::path::PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .filter(|dir| dir.starts_with('/'))
        .map(|dir| std::path::Path::new(dir).join(name))
        .find(|path| {
            std::fs::metadata(path)
                .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        })
}

pub fn home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/".into())
}

/// $XDG_STATE_HOME/tatami; an unset or relative variable means ~/.local/state.
pub fn state_dir() -> std::path::PathBuf {
    let base = std::env::var("XDG_STATE_HOME")
        .ok()
        .filter(|dir| dir.starts_with('/'))
        .unwrap_or_else(|| format!("{}/.local/state", home()));
    std::path::Path::new(&base).join("tatami")
}

/// Processes whose command name matches, from /proc.
pub fn pids_named(names: &[&str]) -> Vec<i32> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return vec![];
    };
    entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.parse::<i32>().ok())
        .filter(|pid| {
            std::fs::read_to_string(format!("/proc/{pid}/comm"))
                .is_ok_and(|comm| names.contains(&comm.trim()))
        })
        // An exited child not yet reaped still has its name: not running.
        .filter(|pid| {
            std::fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|stat| !is_zombie(&stat))
        })
        .collect()
}

/// The state in /proc/PID/stat, after the parenthesized name (which may
/// itself contain spaces and parentheses).
fn is_zombie(stat: &str) -> bool {
    stat.rsplit_once(") ")
        .is_some_and(|(_, rest)| rest.starts_with('Z'))
}

pub fn terminate(pids: &[i32]) {
    for pid in pids {
        unsafe {
            libc::kill(*pid, libc::SIGTERM);
        }
    }
}

/// Ask Waybar to refresh a custom module bound to `signal` (SIGRTMIN+n).
pub fn signal_waybar(offset: i32) {
    for pid in pids_named(&["waybar", ".waybar-wrapped"]) {
        unsafe {
            libc::kill(pid, libc::SIGRTMIN() + offset);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zombies_are_not_running() {
        assert!(is_zombie("1479 (gpu-screen-reco) Z 1468 1468"));
        assert!(!is_zombie("1468 (slurp) S 1 1468"));
        assert!(!is_zombie("7 (a) Z) R 1 7"));
    }
}
