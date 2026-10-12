//! What's installed plus what's released → what to tell the user.

use serde::{Deserialize, Serialize};

use crate::version::{self, Version};

/// How an installed copy got onto the machine, which decides how to update and remove it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstallSource {
    /// Installed by this launcher (it keeps a record and can update or remove it).
    Launcher,
    /// A macOS `.app` the user put in Applications.
    MacApp,
    /// A Windows Installer (MSI) install.
    Msi,
    /// A Debian package.
    Deb,
    /// An RPM package.
    Rpm,
    /// An Arch Linux package (from the AUR: `<app>`, `<app>-bin` or `<app>-git`).
    Pacman,
    /// A Flatpak.
    Flatpak,
    /// An AppImage file found on disk.
    AppImage,
    /// A binary found on `PATH` or in a system prefix.
    Binary,
}

impl InstallSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Launcher => "ArtCraft Launcher",
            Self::MacApp => "Applications folder",
            Self::Msi => "Windows Installer",
            Self::Deb => "Debian package",
            Self::Rpm => "RPM package",
            Self::Pacman => "Arch package (AUR)",
            Self::Flatpak => "Flatpak",
            Self::AppImage => "AppImage",
            Self::Binary => "System binary",
        }
    }

    /// Whether the launcher may replace or remove this copy itself. Packages owned by a system
    /// package manager (deb, rpm, Flatpak) are updated through that manager instead.
    pub fn launcher_can_update(self) -> bool {
        matches!(self, Self::Launcher | Self::MacApp | Self::Msi | Self::AppImage)
    }
}

/// An installed copy of an app.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Installed {
    /// `None` when the copy is there but its version couldn't be read.
    pub version: Option<Version>,
    pub source: InstallSource,
    /// Where it lives: the `.app`, the exe, the AppImage, or the binary of a package.
    pub location: String,
    /// The package that installed it, for package-manager installs (`photocraft-bin`).
    #[serde(default)]
    pub package: Option<String>,
}

/// The status shown on an app's card or row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppStatus {
    /// Still waiting for the first answer.
    Checking,
    /// Released and not installed.
    Available,
    /// Installed, and the installed version is the latest.
    UpToDate,
    /// Installed, and a newer version is out.
    UpdateAvailable,
    /// Installed, but the latest release couldn't be checked (offline, rate-limited).
    Installed,
    /// Not installed and nothing is published for this platform.
    Unavailable,
    /// The app has no releases yet.
    NoRelease,
}

impl AppStatus {
    pub fn is_installed(&self) -> bool {
        matches!(self, Self::UpToDate | Self::UpdateAvailable | Self::Installed)
    }
}

/// What the latest-release lookup produced for an app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Latest<'a> {
    Pending,
    Failed,
    NoRelease,
    Found { version: Option<&'a Version>, has_asset: bool },
}

pub fn status(installed: Option<&Installed>, latest: Latest<'_>) -> AppStatus {
    match (installed, latest) {
        (None, Latest::Pending) => AppStatus::Checking,
        (None, Latest::NoRelease) => AppStatus::NoRelease,
        (None, Latest::Failed) => AppStatus::Unavailable,
        (None, Latest::Found { has_asset, .. }) => {
            if has_asset {
                AppStatus::Available
            } else {
                AppStatus::Unavailable
            }
        }
        (Some(_), Latest::Pending | Latest::Failed | Latest::NoRelease) => AppStatus::Installed,
        (Some(inst), Latest::Found { version: latest, .. }) => match (latest, inst.version.as_ref()) {
            (Some(l), Some(i)) if version::is_update(l, i) => AppStatus::UpdateAvailable,
            (Some(_), Some(_)) => AppStatus::UpToDate,
            // Version unknown on one side: we can't promise "up to date".
            _ => AppStatus::Installed,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inst(v: &str) -> Installed {
        Installed { version: version::parse(v), source: InstallSource::Launcher, location: String::new(), package: None }
    }

    #[test]
    fn every_install_source_has_a_stable_label_and_update_policy() {
        let expected = [
            (InstallSource::Launcher, "ArtCraft Launcher", true),
            (InstallSource::MacApp, "Applications folder", true),
            (InstallSource::Msi, "Windows Installer", true),
            (InstallSource::Deb, "Debian package", false),
            (InstallSource::Rpm, "RPM package", false),
            (InstallSource::Pacman, "Arch package (AUR)", false),
            (InstallSource::Flatpak, "Flatpak", false),
            (InstallSource::AppImage, "AppImage", true),
            (InstallSource::Binary, "System binary", false),
        ];
        for (source, label, may_update) in expected {
            assert_eq!(source.label(), label, "{source:?}");
            assert_eq!(source.launcher_can_update(), may_update, "{source:?}");
            assert_eq!(serde_json::from_str::<InstallSource>(&serde_json::to_string(&source).unwrap()).unwrap(), source);
        }
    }

    #[test]
    fn installed_json_accepts_older_records_without_optional_package() {
        let previous = r#"{"version":"0.4.1","source":"Deb","location":"/usr/bin/photocraft"}"#;
        let installed: Installed = serde_json::from_str(previous).unwrap();
        assert_eq!(installed.version, version::parse("0.4.1"));
        assert_eq!(installed.source, InstallSource::Deb);
        assert_eq!(installed.location, "/usr/bin/photocraft");
        assert_eq!(installed.package, None);
        let newer = Installed { package: Some("photocraft-bin".into()), ..installed };
        assert_eq!(serde_json::from_str::<Installed>(&serde_json::to_string(&newer).unwrap()).unwrap(), newer);
    }

    #[test]
    fn status_preserves_installed_state_across_offline_and_unknown_releases() {
        let known = inst("0.4.0");
        let unknown = inst("invalid version");
        let latest = Version::new(0, 5, 0);

        for installed in [&known, &unknown] {
            for result in [Latest::Pending, Latest::Failed, Latest::NoRelease] {
                assert_eq!(status(Some(installed), result), AppStatus::Installed);
            }
            assert_eq!(status(Some(installed), Latest::Found { version: None, has_asset: true }), AppStatus::Installed);
        }
        assert_eq!(status(Some(&known), Latest::Found { version: Some(&latest), has_asset: true }), AppStatus::UpdateAvailable);
        assert_eq!(status(Some(&unknown), Latest::Found { version: Some(&latest), has_asset: true }), AppStatus::Installed);
        for state in [
            AppStatus::Checking,
            AppStatus::Available,
            AppStatus::UpToDate,
            AppStatus::UpdateAvailable,
            AppStatus::Installed,
            AppStatus::Unavailable,
            AppStatus::NoRelease,
        ] {
            let expected = matches!(state, AppStatus::UpToDate | AppStatus::UpdateAvailable | AppStatus::Installed);
            assert_eq!(state.is_installed(), expected, "{state:?}");
        }
    }

    #[test]
    fn status_covers_every_case() {
        let v5 = Version::new(0, 5, 0);
        let found = |has_asset| Latest::Found { version: Some(&v5), has_asset };
        assert_eq!(status(None, Latest::Pending), AppStatus::Checking);
        assert_eq!(status(None, found(true)), AppStatus::Available);
        assert_eq!(status(None, found(false)), AppStatus::Unavailable);
        assert_eq!(status(None, Latest::NoRelease), AppStatus::NoRelease);
        assert_eq!(status(Some(&inst("0.4.0")), found(true)), AppStatus::UpdateAvailable);
        assert_eq!(status(Some(&inst("0.5.0")), found(true)), AppStatus::UpToDate);
        // Newer than the latest release (a local build): not an update, still current.
        assert_eq!(status(Some(&inst("0.6.0")), found(true)), AppStatus::UpToDate);
        assert_eq!(status(Some(&inst("0.4.0")), Latest::Failed), AppStatus::Installed);
        assert_eq!(status(Some(&inst("garbage")), found(true)), AppStatus::Installed);
        assert!(AppStatus::UpdateAvailable.is_installed() && !AppStatus::Available.is_installed());
    }
}
