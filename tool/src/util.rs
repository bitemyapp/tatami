// SPDX-License-Identifier: GPL-3.0-or-later
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

pub fn notify(summary: &str, body: &str) {
    let mut args = vec!["-u", "low", summary];
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
        .collect()
}

/// Ask Waybar to refresh a custom module bound to `signal` (SIGRTMIN+n).
pub fn signal_waybar(offset: i32) {
    for pid in pids_named(&["waybar", ".waybar-wrapped"]) {
        unsafe {
            libc::kill(pid, libc::SIGRTMIN() + offset);
        }
    }
}
