//! Where the launcher keeps its own files.

use std::io::Write;
use std::path::PathBuf;

/// The launcher's folders.
#[derive(Clone, Debug)]
pub struct Paths {
    /// Records of what the launcher installed, and tarball installs.
    pub data: PathBuf,
    /// Saved release answers and in-progress downloads.
    pub cache: PathBuf,
}

const DIR: &str = if cfg!(any(target_os = "macos", target_os = "windows")) { "ArtCraft Launcher" } else { "artcraft-launcher" };

impl Paths {
    pub fn new() -> Result<Self, String> {
        if let Some(root) = portable_root() {
            return Ok(Self::under(&root));
        }
        let data = dirs::data_local_dir().ok_or("no per-user data folder on this system")?.join(DIR);
        let cache = dirs::cache_dir().ok_or("no per-user cache folder on this system")?.join(DIR);
        Ok(Self { data, cache })
    }

    /// For tests: everything under one folder.
    pub fn under(root: &std::path::Path) -> Self {
        Self { data: root.join("data"), cache: root.join("cache") }
    }

    pub fn downloads(&self) -> PathBuf {
        self.cache.join("downloads")
    }

    pub fn release_cache(&self) -> PathBuf {
        self.cache.join("releases.json")
    }

    /// The last answer from ArtCraft's release feed, for an instant first paint and as a fallback.
    pub fn feed_cache(&self) -> PathBuf {
        self.cache.join("feed.json")
    }

    /// How long to hold off calling ArtCraft's API (after failures or HTTP 429), across restarts.
    pub fn feed_throttle(&self) -> PathBuf {
        self.cache.join("feed-throttle.json")
    }

    /// A picture of an app's details page, saved after its first download: under the revision of the
    /// pictures repository it was fetched from, so a new revision never reads an old picture.
    pub fn screenshot(&self, app: &str, key: &str) -> PathBuf {
        self.cache.join("screenshots").join(artcraft_launcher_core::details::SCREENSHOTS_REV).join(app).join(format!("{key}.jpg"))
    }

    pub fn manifest(&self) -> PathBuf {
        self.data.join("installed.json")
    }

    /// The launcher's own log files (`artcraft-launcher.log` and the previous runs').
    pub fn logs(&self) -> PathBuf {
        self.data.join("logs")
    }

    /// Where tarball installs (FreeBSD, Linux without AppImage) are unpacked: `<data>/apps/<app>`.
    pub fn apps(&self) -> PathBuf {
        self.data.join("apps")
    }

    /// The folder an app unpacked in [`Self::apps`] sees as its home: `<data>/apps/<app>/home`.
    /// A Craft App keeps its settings in the XDG folders (`~/.config`, `~/.local/share`,
    /// `~/.cache`), so pointing `HOME` and those variables here keeps every app's configuration
    /// beside it, inside the folder the launcher owns.
    #[cfg(target_os = "linux")]
    pub fn app_home(&self, id: &str) -> PathBuf {
        self.apps().join(id).join("home")
    }
}

/// Writes `bytes` to `path` via a temporary file and a rename, so a crash never leaves half a file.
pub fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    let dir = path.parent().ok_or("no parent folder")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("couldn't create {} ({e})", dir.display()))?;
    // Each writer owns a securely created file. A fixed .tmp path would let concurrent
    // saves clobber one another, or follow a pre-existing symlink before the rename.
    let mut tmp = tempfile::NamedTempFile::new_in(dir).map_err(|e| format!("couldn't create a temporary file in {} ({e})", dir.display()))?;
    tmp.write_all(bytes).map_err(|e| format!("couldn't write the temporary file ({e})"))?;
    tmp.as_file().sync_all().map_err(|e| format!("couldn't sync the temporary file ({e})"))?;
    tmp.persist(path).map_err(|e| format!("couldn't replace {} ({e})", path.display()))?;
    Ok(())
}

/// Portable mode: a `portable.txt` beside the executable keeps everything in
/// `ArtCraftLauncherData` next to it (the Windows portable zip ships one).
pub fn portable_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    dir.join("portable.txt").is_file().then(|| dir.join("ArtCraftLauncherData"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_ignores_a_legacy_temp_directory_and_replaces_existing_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("releases.json");
        std::fs::write(&path, b"before").unwrap();
        let legacy_tmp = path.with_extension("tmp");
        std::fs::create_dir(&legacy_tmp).unwrap();

        write_atomic(&path, b"after").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"after");
        assert!(legacy_tmp.is_dir(), "unrelated temporary path must not be touched");

        write_atomic(&path, b"after a second write").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"after a second write");
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_cannot_follow_a_preexisting_legacy_temp_symlink() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("feed.json");
        let guarded = dir.path().join("keep.txt");
        std::fs::write(&guarded, b"preserve this").unwrap();
        symlink(&guarded, path.with_extension("tmp")).unwrap();

        write_atomic(&path, b"replacement").unwrap();
        assert_eq!(std::fs::read(&guarded).unwrap(), b"preserve this");
        assert_eq!(std::fs::read(&path).unwrap(), b"replacement");
    }

    #[test]
    fn concurrent_writers_each_commit_a_complete_file() {
        use std::sync::{Arc, Barrier};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.json");
        let barrier = Arc::new(Barrier::new(3));
        let first = vec![b'a'; 32 * 1024];
        let second = vec![b'b'; 32 * 1024];
        let handles: Vec<_> = [first.clone(), second.clone()]
            .into_iter()
            .map(|bytes| {
                let barrier = Arc::clone(&barrier);
                let path = path.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    write_atomic(&path, &bytes)
                })
            })
            .collect();
        barrier.wait();
        for handle in handles {
            handle.join().unwrap().unwrap();
        }
        let actual = std::fs::read(&path).unwrap();
        assert!(actual == first || actual == second, "an atomic commit must contain one complete value");
    }
}
