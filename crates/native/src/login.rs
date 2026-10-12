//! "Start ArtCraft Launcher when I log in".
//!
//! - macOS: a LaunchAgent in `~/Library/LaunchAgents`.
//! - Windows: a value under `HKCU\…\CurrentVersion\Run`.
//! - Linux and FreeBSD: an XDG autostart entry in `~/.config/autostart`.
//!
//! The launcher starts hidden in the tray when it's started this way (`--hidden`).

use std::path::PathBuf;

#[cfg_attr(windows, allow(dead_code))]
const ID: &str = "ai.storyteller.artcraft-launcher";
/// The command-line flag a login start passes.
pub const HIDDEN_FLAG: &str = "--hidden";

/// The program to start at login: the AppImage when running from one, else this executable.
fn program() -> Result<PathBuf, String> {
    if let Some(appimage) = std::env::var_os("APPIMAGE") {
        return Ok(PathBuf::from(appimage));
    }
    std::env::current_exe().map_err(|e| format!("couldn't find this program ({e})"))
}

#[cfg(target_os = "macos")]
fn agent() -> Result<PathBuf, String> {
    Ok(dirs::home_dir().ok_or("no home folder")?.join("Library/LaunchAgents").join(format!("{ID}.plist")))
}

#[cfg(target_os = "macos")]
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub fn is_enabled() -> bool {
    #[cfg(target_os = "macos")]
    return agent().is_ok_and(|p| p.exists());
    #[cfg(windows)]
    return windows_run().and_then(|k| k.get_value::<String, _>("ArtCraft Launcher").ok()).is_some();
    #[cfg(not(any(target_os = "macos", windows)))]
    return autostart().is_ok_and(|p| p.exists());
}

pub fn set_enabled(on: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let path = agent()?;
        if !on {
            return remove(&path);
        }
        // Inside a bundle, `open -a` starts the app the way Finder does (one instance, Dock icon).
        let exe = program()?;
        let bundle = exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app")).map(PathBuf::from);
        let args: Vec<String> = match bundle {
            Some(b) => vec!["/usr/bin/open".into(), "-a".into(), b.display().to_string(), "--args".into(), HIDDEN_FLAG.into()],
            None => vec![exe.display().to_string(), HIDDEN_FLAG.into()],
        };
        let args_xml: String = args.iter().map(|a| format!("    <string>{}</string>\n", xml_escape(a))).collect();
        let plist = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\">\n<dict>\n  <key>Label</key>\n  <string>{ID}</string>\n  <key>ProgramArguments</key>\n  <array>\n{args_xml}  </array>\n  <key>RunAtLoad</key>\n  <true/>\n</dict>\n</plist>\n"
        );
        crate::paths::write_atomic(&path, plist.as_bytes())
    }
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
            .map_err(|e| format!("couldn't change the startup list ({e})"))?;
        if on {
            let value = format!("\"{}\" {HIDDEN_FLAG}", program()?.display());
            key.set_value("ArtCraft Launcher", &value).map_err(|e| format!("couldn't change the startup list ({e})"))
        } else {
            match key.delete_value("ArtCraft Launcher") {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(format!("couldn't change the startup list ({e})")),
            }
        }
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        let path = autostart()?;
        if !on {
            return remove(&path);
        }
        let entry = autostart_entry(&program()?)?;
        crate::paths::write_atomic(&path, entry.as_bytes())
    }
}

#[cfg(windows)]
fn windows_run() -> Option<winreg::RegKey> {
    winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run").ok()
}

/// Encodes a path as one Exec argument according to XDG Desktop Entry:
/// quote reserved characters, escape the whole string value's backslashes,
/// and double literal percent signs so they are not interpreted as field codes.
/// Reject paths that cannot be represented without changing the executable.
#[cfg(not(any(target_os = "macos", windows)))]
fn autostart_command(exe: &std::path::Path) -> Result<String, String> {
    let path = exe.to_str().ok_or("the executable path isn't valid UTF-8")?;
    if !exe.is_absolute() || path.chars().any(|c| c.is_control() || c == '=') {
        return Err("the executable path cannot be represented in an XDG Exec entry".into());
    }
    let mut quoted = String::new();
    for c in path.chars() {
        match c {
            '%' => quoted.push_str("%%"),
            '\\' | '"' | '
    Ok(dirs::config_dir().ok_or("no config folder")?.join("autostart").join(format!("{ID}.desktop")))
}

#[cfg(not(windows))]
fn remove(path: &std::path::Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("couldn't remove {} ({e})", path.display())),
    }
}
 | '\u{60}' => {
                quoted.push('\\');
                quoted.push(c);
            }
            _ => quoted.push(c),
        }
    }
    // The Desktop Entry string parser consumes a first layer of backslashes,
    // then the Exec parser consumes the quoting layer.
    Ok(format!("\"{}\" {HIDDEN_FLAG}", quoted.replace('\\', "\\\\")))
}

#[cfg(not(any(target_os = "macos", windows)))]
fn autostart_entry(exe: &std::path::Path) -> Result<String, String> {
    let command = autostart_command(exe)?;
    Ok(format!(
        "[Desktop Entry]\nType=Application\nName=ArtCraft Launcher\nComment=Install and update the Craft App Suite\nExec={command}\nIcon={ID}\nTerminal=false\nX-GNOME-Autostart-enabled=true\n"
    ))
}

#[cfg(not(any(target_os = "macos", windows)))]
fn autostart() -> Result<PathBuf, String> {
    Ok(dirs::config_dir().ok_or("no config folder")?.join("autostart").join(format!("{ID}.desktop")))
}

#[cfg(not(windows))]
fn remove(path: &std::path::Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("couldn't remove {} ({e})", path.display())),
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(any(target_os = "macos", windows)))]
    use super::*;

    #[cfg(not(any(target_os = "macos", windows)))]
    #[test]
    fn xdg_login_entry_preserves_spaces_and_hidden_flag() {
        let entry = autostart_entry(std::path::Path::new("/opt/My ArtCraft Apps/launcher")).unwrap();
        assert!(entry.starts_with("[Desktop Entry]\nType=Application\n"));
        assert!(entry.contains("Exec=\"/opt/My ArtCraft Apps/launcher\" --hidden\n"));
        assert!(entry.contains("X-GNOME-Autostart-enabled=true\n"));
        assert_eq!(entry.lines().filter(|line| line.starts_with("Exec=")).count(), 1);
    }

    #[cfg(not(any(target_os = "macos", windows)))]
    #[test]
    fn xdg_login_exec_escapes_reserved_chars_and_field_codes() {
        let command = autostart_command(std::path::Path::new("/tmp/a$b`c\\d\"e%f")).unwrap();
        assert!(command.contains(r"a\\$b"), "{command:?}");
        assert!(command.contains(r"b\\`c"), "{command:?}");
        assert!(command.contains(r"c\\\\d"), "{command:?}");
        assert!(command.contains("e%%f"), "{command:?}");
        assert!(command.contains(r#"\\\"e"#), "{command:?}");
        assert!(command.ends_with("\" --hidden"));
    }

    #[cfg(not(any(target_os = "macos", windows)))]
    #[test]
    fn xdg_login_rejects_line_injection_and_invalid_executables() {
        for invalid in [
            "/opt/launcher\nHidden=true",
            "/opt/launcher\rExec=other",
            "/opt/launcher\tbad",
            "/opt/launcher=bad",
            "relative/launcher",
        ] {
            assert!(autostart_entry(std::path::Path::new(invalid)).is_err(), "{invalid:?}");
        }
        assert!(autostart_entry(std::path::Path::new("")).is_err());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    #[test]
    fn xdg_login_rejects_non_utf8_paths_instead_of_mangling_them() {
        use std::os::unix::ffi::OsStrExt;

        let invalid = std::ffi::OsStr::from_bytes(b"/tmp/app-\xff");
        assert!(autostart_entry(std::path::Path::new(invalid)).is_err());
    }
}
