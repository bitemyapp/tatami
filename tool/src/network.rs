// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wi-Fi in a Walker list: the bar's network icon, Super+Ctrl+W and
//! Setup › Network. Omarchy 4 has a Wi-Fi panel there instead of a terminal
//! tool; this is that panel over NetworkManager. Passwords are asked for in
//! Walker and reach nmcli in a file only the user can read, never on a
//! command line; the connections it creates keep their key root-only, like
//! the installer's.
use crate::util;
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

/// One network from `nmcli -t -f IN-USE,SSID,SECURITY,SIGNAL device wifi list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Network {
    pub ssid: String,
    pub security: String,
    pub signal: u8,
    pub active: bool,
}

/// How a network is joined.
#[derive(Debug, PartialEq, Eq)]
pub enum Join {
    /// No key: open and enhanced-open (OWE) networks.
    Open,
    /// A personal password, with NetworkManager's key management for it.
    Password(&'static str),
    /// WEP and enterprise (802.1X) networks need Network settings.
    Settings,
}

impl Network {
    pub fn join(&self) -> Join {
        let security = self.security.as_str();
        if security.is_empty() || security == "--" || security == "OWE" {
            Join::Open
        } else if security.contains("802.1X") || security.contains("WEP") {
            Join::Settings
        } else if security.contains("WPA3")
            && !security.contains("WPA2")
            && !security.contains("WPA1")
        {
            Join::Password("sae")
        } else {
            Join::Password("wpa-psk")
        }
    }
}

/// Fields of a terse nmcli line: `:` separates them, `\:` and `\\` escape.
pub fn fields(line: &str) -> Vec<String> {
    let mut fields = vec![String::new()];
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    fields.last_mut().unwrap().push(next);
                }
            }
            ':' => fields.push(String::new()),
            c => fields.last_mut().unwrap().push(c),
        }
    }
    fields
}

/// Visible networks, one per name: the active one first, then by signal.
/// Hidden networks (no name) are left to Network settings.
pub fn networks(list: &str) -> Vec<Network> {
    let mut found: Vec<Network> = Vec::new();
    for line in list.lines() {
        let [in_use, ssid, security, signal] = fields(line).try_into().unwrap_or_default();
        if ssid.is_empty() {
            continue;
        }
        let network = Network {
            active: in_use == "*",
            signal: signal.parse().unwrap_or(0),
            ssid,
            security,
        };
        match found.iter_mut().find(|known| known.ssid == network.ssid) {
            Some(known) => {
                known.active |= network.active;
                if network.signal > known.signal {
                    known.signal = network.signal;
                    known.security = network.security;
                }
            }
            None => found.push(network),
        }
    }
    found.sort_by_key(|network| (!network.active, std::cmp::Reverse(network.signal)));
    found
}

/// The bar's signal glyphs, with their padlock versions for secured networks.
pub fn glyph(network: &Network) -> char {
    const OPEN: [char; 5] = [
        '\u{f092f}',
        '\u{f091f}',
        '\u{f0922}',
        '\u{f0925}',
        '\u{f0928}',
    ];
    const LOCKED: [char; 5] = [
        '\u{f092c}',
        '\u{f0921}',
        '\u{f0924}',
        '\u{f0927}',
        '\u{f092a}',
    ];
    let level = usize::from(network.signal.min(100) / 20).min(4);
    if network.join() == Join::Open {
        OPEN[level]
    } else {
        LOCKED[level]
    }
}

const WIFI_ON: &str = "\u{f05a9}  Turn Wi-Fi on";
const WIFI_OFF: &str = "\u{f05aa}  Turn Wi-Fi off";
const SETTINGS: &str = "\u{f0493}  Network settings…";

enum Row {
    Network(Network),
    Disconnect(String),
    Radio(bool),
    Settings,
}

/// Networks first, so Return on opening never turns anything off; then
/// disconnecting, the Wi-Fi switch and Network settings.
fn rows(enabled: bool, networks: Vec<Network>) -> Vec<(String, Row)> {
    if !enabled {
        return vec![
            (WIFI_ON.to_owned(), Row::Radio(true)),
            (SETTINGS.to_owned(), Row::Settings),
        ];
    }
    let active = networks
        .iter()
        .find(|network| network.active)
        .map(|network| network.ssid.clone());
    let mut rows: Vec<(String, Row)> = networks
        .into_iter()
        .map(|network| {
            let check = if network.active { "  \u{f012c}" } else { "" };
            let label = format!("{}  {}{check}", glyph(&network), network.ssid);
            (label, Row::Network(network))
        })
        .collect();
    if let Some(ssid) = active {
        rows.push((
            format!("\u{f05aa}  Disconnect from {ssid}"),
            Row::Disconnect(ssid),
        ));
    }
    rows.push((WIFI_OFF.to_owned(), Row::Radio(false)));
    rows.push((SETTINGS.to_owned(), Row::Settings));
    rows
}

fn wifi_device() -> Option<String> {
    util::output("nmcli", &["-t", "-f", "DEVICE,TYPE", "device"])
        .ok()?
        .lines()
        .map(fields)
        .find(|fields| fields.get(1).map(String::as_str) == Some("wifi"))
        .map(|fields| fields[0].clone())
}

/// A saved Wi-Fi connection for this network name, by UUID.
fn saved(ssid: &str) -> Option<String> {
    let list = util::output("nmcli", &["-t", "-f", "UUID,TYPE", "connection", "show"]).ok()?;
    list.lines()
        .map(fields)
        .filter(|fields| fields.get(1).map(String::as_str) == Some("802-11-wireless"))
        .map(|fields| fields[0].clone())
        .find(|uuid| {
            util::output(
                "nmcli",
                &[
                    "-g",
                    "802-11-wireless.ssid",
                    "connection",
                    "show",
                    "uuid",
                    uuid,
                ],
            )
            .is_ok_and(|name| name.trim_end_matches('\n') == ssid)
        })
}

/// Run nmcli; the error is its message.
fn nmcli(args: &[&str]) -> Result<(), String> {
    let out = Command::new("nmcli")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| error.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&out.stderr);
        Err(message.trim().trim_start_matches("Error: ").to_owned())
    }
}

/// A Wi-Fi key for `nmcli … passwd-file`, in the user's runtime directory
/// (memory only, mode 0700) and removed again when dropped.
struct Secret(PathBuf);

impl Secret {
    fn new(password: &str) -> Option<Secret> {
        use std::os::unix::fs::OpenOptionsExt;
        let dir = std::env::var("XDG_RUNTIME_DIR")
            .ok()
            .filter(|dir| dir.starts_with('/'))?;
        let path = PathBuf::from(dir).join(format!("tatami-wifi-{}", std::process::id()));
        let _ = fs::remove_file(&path);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .ok()?;
        let secret = Secret(path);
        file.write_all(format!("802-11-wireless-security.psk:{password}\n").as_bytes())
            .ok()?;
        Some(secret)
    }

    fn path(&self) -> &str {
        self.0.to_str().unwrap_or_default()
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// What to tell the user when joining failed.
fn reason(error: &str) -> &str {
    if error.contains("Timeout") || error.contains("Secrets were required") {
        "Check the password and try again"
    } else {
        error
    }
}

/// Walker's password prompt; None when cancelled.
fn ask_password(ssid: &str) -> Option<String> {
    let prompt = format!("Password for {ssid}…");
    let password = util::filter("walker", &["--password", "-p", &prompt], b"").ok()?;
    let password = password.trim_end_matches('\n').to_owned();
    (!password.is_empty()).then_some(password)
}

const ICON: &str = "\u{f05a9}";

fn connected(ssid: &str) {
    util::notify(ICON, &format!("Connected to {ssid}"), "");
}

fn failed(ssid: &str, why: &str) {
    util::notify("\u{f05aa}", &format!("Could not connect to {ssid}"), why);
}

/// Join with a password: a new connection made with the key on stdin, or
/// the saved one given a new key. A new connection that fails is removed.
fn join_with_password(device: &str, network: &Network, key_mgmt: &str, existing: Option<String>) {
    let Some(password) = ask_password(&network.ssid) else {
        return;
    };
    let Some(secret) = Secret::new(&password) else {
        return failed(&network.ssid, "Cannot hand the password to NetworkManager");
    };
    let (uuid, created) = match existing {
        Some(uuid) => (uuid, false),
        None => {
            let name = network.ssid.as_str();
            // Our own UUID: nmcli's report of the one it chose is prose.
            let Some(uuid) = fs::read_to_string("/proc/sys/kernel/random/uuid")
                .ok()
                .map(|uuid| uuid.trim().to_owned())
            else {
                return failed(name, "Cannot create the connection");
            };
            let added = nmcli(&[
                "connection",
                "add",
                "type",
                "wifi",
                "con-name",
                name,
                "ifname",
                device,
                "ssid",
                name,
                "connection.uuid",
                &uuid,
                "wifi-sec.key-mgmt",
                key_mgmt,
            ]);
            match added {
                Ok(()) => (uuid, true),
                Err(error) => return failed(name, &error),
            }
        }
    };
    util::notify(ICON, &format!("Connecting to {}…", network.ssid), "");
    match nmcli(&[
        "--wait",
        "30",
        "connection",
        "up",
        "uuid",
        &uuid,
        "passwd-file",
        secret.path(),
    ]) {
        Ok(()) => connected(&network.ssid),
        Err(error) => {
            if created {
                let _ = nmcli(&["connection", "delete", "uuid", &uuid]);
            }
            failed(&network.ssid, reason(&error));
        }
    }
}

fn join(device: &str, network: &Network) {
    if network.active {
        return connected(&network.ssid);
    }
    let existing = saved(&network.ssid);
    match network.join() {
        Join::Settings if existing.is_none() => {
            util::notify(
                ICON,
                &format!("{} needs Network settings", network.ssid),
                "Enterprise and WEP networks are set up there",
            );
            settings();
        }
        Join::Password(key_mgmt) if existing.is_none() => {
            join_with_password(device, network, key_mgmt, None)
        }
        join => {
            // Saved and open networks: NetworkManager has what it needs.
            util::notify(ICON, &format!("Connecting to {}…", network.ssid), "");
            let result = match &existing {
                Some(uuid) => nmcli(&["--wait", "30", "connection", "up", "uuid", uuid]),
                None => nmcli(&[
                    "--wait",
                    "30",
                    "device",
                    "wifi",
                    "connect",
                    &network.ssid,
                    "ifname",
                    device,
                ]),
            };
            match (result, join) {
                (Ok(()), _) => connected(&network.ssid),
                // The saved key is wrong or held by another desktop's wallet.
                (Err(_), Join::Password(key_mgmt)) => {
                    join_with_password(device, network, key_mgmt, existing)
                }
                (Err(why), _) => failed(&network.ssid, reason(&why)),
            }
        }
    }
}

fn settings() {
    util::launch("nm-connection-editor", &[]);
}

pub fn show() {
    crate::menu::after_menu();
    let Some(device) = wifi_device() else {
        // Wired-only machines: the bar's network icon opens the settings.
        settings();
        return;
    };
    let enabled =
        util::output("nmcli", &["radio", "wifi"]).is_ok_and(|state| state.trim() == "enabled");
    let networks = if enabled {
        let list = util::output(
            "nmcli",
            &[
                "-t",
                "-f",
                "IN-USE,SSID,SECURITY,SIGNAL",
                "device",
                "wifi",
                "list",
                "ifname",
                &device,
            ],
        )
        .unwrap_or_default();
        networks(&list)
    } else {
        Vec::new()
    };
    let rows = rows(enabled, networks);
    let labels: Vec<String> = rows.iter().map(|(label, _)| label.clone()).collect();
    let current = rows
        .iter()
        .position(|(_, row)| matches!(row, Row::Network(network) if network.active));
    let Some(index) = crate::menu::pick_index("Wi-Fi", &labels, current, "420") else {
        return;
    };
    match &rows[index].1 {
        Row::Radio(on) => {
            let state = if *on { "on" } else { "off" };
            if nmcli(&["radio", "wifi", state]).is_ok() {
                util::notify(ICON, &format!("Wi-Fi is {state}"), "");
            }
        }
        Row::Network(network) => join(&device, network),
        Row::Disconnect(ssid) => {
            if nmcli(&["device", "disconnect", &device]).is_ok() {
                util::notify("\u{f05aa}", &format!("Disconnected from {ssid}"), "");
            }
        }
        Row::Settings => settings(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn network(ssid: &str, security: &str, signal: u8, active: bool) -> Network {
        Network {
            ssid: ssid.into(),
            security: security.into(),
            signal,
            active,
        }
    }

    #[test]
    fn terse_fields_unescape_colons_and_backslashes() {
        assert_eq!(
            fields(r" :Cafe\: Open \\ Wi-Fi::100"),
            [" ", r"Cafe: Open \ Wi-Fi", "", "100"]
        );
        assert_eq!(fields("*:Home:WPA2:57"), ["*", "Home", "WPA2", "57"]);
    }

    #[test]
    fn networks_are_unique_active_first_then_strongest() {
        let list = " :Cafe::40\n :Home:WPA2:57\n*:Office:WPA2 WPA3:30\n :Home:WPA2:80\n :::90\n";
        assert_eq!(
            networks(list),
            [
                network("Office", "WPA2 WPA3", 30, true),
                network("Home", "WPA2", 80, false),
                network("Cafe", "", 40, false),
            ]
        );
    }

    #[test]
    fn security_decides_how_to_join() {
        assert_eq!(network("a", "", 1, false).join(), Join::Open);
        assert_eq!(network("a", "OWE", 1, false).join(), Join::Open);
        assert_eq!(
            network("a", "WPA1 WPA2", 1, false).join(),
            Join::Password("wpa-psk")
        );
        assert_eq!(
            network("a", "WPA2 WPA3", 1, false).join(),
            Join::Password("wpa-psk")
        );
        assert_eq!(network("a", "WPA3", 1, false).join(), Join::Password("sae"));
        assert_eq!(network("a", "WPA2 802.1X", 1, false).join(), Join::Settings);
        assert_eq!(network("a", "WEP", 1, false).join(), Join::Settings);
    }

    #[test]
    fn rows_list_networks_first_then_actions() {
        let on = rows(
            true,
            vec![
                network("Home", "WPA2", 100, true),
                network("Cafe", "", 10, false),
            ],
        );
        let labels: Vec<&str> = on.iter().map(|(label, _)| label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "\u{f092a}  Home  \u{f012c}",
                "\u{f092f}  Cafe",
                "\u{f05aa}  Disconnect from Home",
                WIFI_OFF,
                SETTINGS
            ]
        );
        let idle = rows(true, vec![network("Cafe", "", 10, false)]);
        assert!(
            !idle
                .iter()
                .any(|(_, row)| matches!(row, Row::Disconnect(_)))
        );
        let off = rows(false, vec![network("Cafe", "", 10, false)]);
        assert_eq!(
            off.iter()
                .map(|(label, _)| label.as_str())
                .collect::<Vec<_>>(),
            [WIFI_ON, SETTINGS]
        );
    }
}
