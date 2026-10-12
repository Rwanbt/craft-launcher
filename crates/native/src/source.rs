//! Where release information comes from.
//!
//! [`GitHubSource`] asks the GitHub REST API (one request per app, revalidated with ETags) and,
//! when the API is out of quota, falls back to github.com's release pages: the
//! `releases/latest` redirect names the tag and `SHA256SUMS.txt` lists every file with its
//! checksum, which costs no API quota at all. A later [`ReleaseSource`] will ask the ArtCraft
//! catalog API for every app in one request.

use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

use artcraft_launcher_core::protocol::{LatestRelease, Source};
use artcraft_launcher_core::release::{self, Release};
use serde::{Deserialize, Serialize};

use crate::http::{Fetched, Http};

/// One answer from a source.
#[derive(Clone, Debug)]
pub struct Answer {
    pub latest: LatestRelease,
    pub source: Source,
}

/// Looks up latest releases. Called from worker threads.
pub trait ReleaseSource: Send + Sync {
    fn latest(&self, repo: &str, prereleases: bool) -> Answer;
    /// The last answer saved for `repo`, for an instant first paint.
    fn cached(&self, repo: &str, prereleases: bool) -> Option<LatestRelease>;
    /// API refused until this time (Unix seconds).
    fn rate_limited_until(&self) -> Option<i64>;
    /// Persist anything worth keeping.
    fn save(&self);
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Entry {
    etag: Option<String>,
    /// `None`: the repo has no releases.
    release: Option<Release>,
    prereleases: bool,
    fetched_at: i64,
}

#[derive(Default, Serialize, Deserialize)]
struct CacheFile {
    entries: BTreeMap<String, Entry>,
}

pub struct GitHubSource {
    http: Http,
    cache: Mutex<CacheFile>,
    cache_path: std::path::PathBuf,
    limited_until: Mutex<Option<i64>>,
}

impl GitHubSource {
    pub fn new(http: Http, cache_path: std::path::PathBuf) -> Self {
        let cache = std::fs::read(&cache_path).ok().and_then(|b| serde_json::from_slice::<CacheFile>(&b).ok()).unwrap_or_default();
        Self { http, cache: Mutex::new(cache), cache_path, limited_until: Mutex::new(None) }
    }

    fn entry(&self, repo: &str, prereleases: bool) -> Option<Entry> {
        let cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        cache.entries.get(repo).filter(|e| e.prereleases == prereleases).cloned()
    }

    fn store(&self, repo: &str, entry: Entry) {
        self.cache.lock().unwrap_or_else(PoisonError::into_inner).entries.insert(repo.to_string(), entry);
    }

    fn limited(&self) -> bool {
        let until = *self.limited_until.lock().unwrap_or_else(PoisonError::into_inner);
        until.is_some_and(|u| u > crate::now())
    }

    fn ask_api(&self, repo: &str, prereleases: bool) -> Result<Option<Answer>, String> {
        let url = if prereleases {
            format!("https://api.github.com/repos/{repo}/releases?per_page=30")
        } else {
            format!("https://api.github.com/repos/{repo}/releases/latest")
        };
        let saved = self.entry(repo, prereleases);
        let found =
            |release: Option<Release>| Answer { latest: release.map_or(LatestRelease::None, LatestRelease::Found), source: Source::GitHubApi };
        match self.http.get_api(&url, saved.as_ref().and_then(|e| e.etag.as_deref()))? {
            Fetched::Body { text, etag } => {
                let release = if prereleases {
                    release::parse_github_release_list(&text, repo, true)?
                } else {
                    Some(release::parse_github_release(&text, repo)?)
                };
                self.store(repo, Entry { etag, release: release.clone(), prereleases, fetched_at: crate::now() });
                Ok(Some(found(release)))
            }
            Fetched::NotModified => match saved {
                Some(e) => {
                    self.store(repo, Entry { fetched_at: crate::now(), ..e.clone() });
                    Ok(Some(found(e.release)))
                }
                None => Err("GitHub said nothing changed, but there's no saved answer".into()),
            },
            Fetched::NotFound => {
                self.store(repo, Entry { etag: None, release: None, prereleases, fetched_at: crate::now() });
                Ok(Some(found(None)))
            }
            Fetched::RateLimited { reset } => {
                *self.limited_until.lock().unwrap_or_else(PoisonError::into_inner) = Some(reset.unwrap_or_else(|| crate::now() + 15 * 60));
                Ok(None)
            }
        }
    }

    /// The quota-free route through github.com (stable releases only).
    fn ask_pages(&self, repo: &str) -> Result<Answer, String> {
        let location = self.http.redirect_target(&format!("https://github.com/{repo}/releases/latest"))?;
        let Some(tag) = location.as_deref().and_then(|l| release::tag_from_redirect(l, repo)) else {
            // No published release: github.com sends us to the release list instead.
            return Ok(Answer { latest: LatestRelease::None, source: Source::GitHubPages });
        };
        if let Some(e) = self.entry(repo, false)
            && let Some(r) = e.release.as_ref().filter(|r| r.tag == tag)
        {
            return Ok(Answer { latest: LatestRelease::Found(r.clone()), source: Source::GitHubPages });
        }
        let sums_url = format!("{}{tag}/SHA256SUMS.txt", release::download_prefix(repo));
        let sums = self.http.get_text(&sums_url)?.map(|t| release::parse_sha256sums(&t)).unwrap_or_default();
        let r = release::release_from_sums(repo, &tag, &sums);
        self.store(repo, Entry { etag: None, release: Some(r.clone()), prereleases: false, fetched_at: crate::now() });
        Ok(Answer { latest: LatestRelease::Found(r), source: Source::GitHubPages })
    }
}

impl ReleaseSource for GitHubSource {
    fn latest(&self, repo: &str, prereleases: bool) -> Answer {
        let api_error = if self.limited() {
            None
        } else {
            match self.ask_api(repo, prereleases) {
                Ok(Some(answer)) => return answer,
                Ok(None) => None,
                Err(e) => Some(e),
            }
        };
        if !prereleases && api_error.is_none() {
            match self.ask_pages(repo) {
                Ok(answer) => return answer,
                Err(e) => log::warn!("{repo}: release pages: {e}"),
            }
        }
        let why = api_error.unwrap_or_else(|| "GitHub's request limit was reached".into());
        match self.cached(repo, prereleases) {
            Some(latest) => Answer { latest, source: Source::Cache },
            None => Answer { latest: LatestRelease::Failed(why), source: Source::Cache },
        }
    }

    fn cached(&self, repo: &str, prereleases: bool) -> Option<LatestRelease> {
        self.entry(repo, prereleases).map(|e| e.release.map_or(LatestRelease::None, LatestRelease::Found))
    }

    fn rate_limited_until(&self) -> Option<i64> {
        let until = *self.limited_until.lock().unwrap_or_else(PoisonError::into_inner);
        until.filter(|u| *u > crate::now())
    }

    fn save(&self) {
        let json = {
            let cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
            serde_json::to_vec(&*cache)
        };
        match json {
            Ok(bytes) => {
                if let Err(e) = crate::paths::write_atomic(&self.cache_path, &bytes) {
                    log::warn!("release cache: {e}");
                }
            }
            Err(e) => log::warn!("release cache: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPO: &str = "storytold/photocraft";

    fn source(path: &std::path::Path) -> GitHubSource {
        // Constructing an HTTP client does not make a network request. The tests
        // exercise only on-disk cache methods, never the live release APIs.
        GitHubSource::new(Http::new().unwrap(), path.to_path_buf())
    }

    fn release(tag: &str) -> Release {
        Release { tag: tag.to_string(), version: artcraft_launcher_core::version::parse(tag), ..Release::default() }
    }

    #[test]
    fn release_cache_survives_restart_with_matching_prerelease_setting() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("releases.json");
        let original = source(&path);
        assert_eq!(original.cached(REPO, false), None);
        let expected = release("v0.5.0");
        original.store(REPO, Entry { etag: Some("\"etag-1\"".into()), release: Some(expected.clone()), prereleases: false, fetched_at: 123 });
        assert_eq!(original.cached(REPO, false), Some(LatestRelease::Found(expected.clone())));
        original.save();

        let persisted: CacheFile = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let record = persisted.entries.get(REPO).unwrap();
        assert_eq!(record.etag.as_deref(), Some("\"etag-1\""));
        assert_eq!(record.fetched_at, 123);
        assert_eq!(record.release.as_ref(), Some(&expected));

        let restored = source(&path);
        assert_eq!(restored.cached(REPO, false), Some(LatestRelease::Found(expected)));
        assert_eq!(restored.cached(REPO, true), None);
        assert_eq!(restored.cached("storytold/wordcraft", false), None);
    }

    #[test]
    fn an_explicit_no_release_answer_differs_from_a_missing_cache_entry() {
        let dir = tempfile::tempdir().unwrap();
        let source = source(&dir.path().join("releases.json"));
        source.store(REPO, Entry { etag: None, release: None, prereleases: false, fetched_at: 9 });
        assert_eq!(source.cached(REPO, false), Some(LatestRelease::None));
        assert_eq!(source.cached(REPO, true), None);
        assert_eq!(source.cached("storytold/wordcraft", false), None);
    }

    #[test]
    fn changing_prerelease_mode_does_not_reuse_an_incompatible_answer() {
        let dir = tempfile::tempdir().unwrap();
        let source = source(&dir.path().join("releases.json"));
        let stable = release("v0.5.0");
        source.store(REPO, Entry { etag: None, release: Some(stable), prereleases: false, fetched_at: 1 });
        assert_eq!(source.cached(REPO, true), None);

        let preview = release("v0.6.0-rc.1");
        source.store(REPO, Entry { etag: None, release: Some(preview.clone()), prereleases: true, fetched_at: 2 });
        assert_eq!(source.cached(REPO, true), Some(LatestRelease::Found(preview)));
        assert_eq!(source.cached(REPO, false), None);
    }

    #[test]
    fn missing_and_corrupt_cache_files_are_treated_as_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("releases.json");
        assert_eq!(source(&path).cached(REPO, false), None);

        for broken in [
            b"{not json".as_slice(),
            br#"{"entries":{"storytold/photocraft":{"etag":42}}}"#.as_slice(),
            br#"{"entries": [1, 2, 3]}"#.as_slice(),
            b"null".as_slice(),
        ] {
            std::fs::write(&path, broken).unwrap();
            assert_eq!(source(&path).cached(REPO, false), None);
        }
    }
}
