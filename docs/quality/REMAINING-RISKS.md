# Quality and production-readiness risks — 2026-10-12

The project README calls the launcher an **early alpha**. A passing isolated PR is not evidence that the whole launcher is production-ready.

1. **BLOCKED / CI evidence:** the fork initially reported no GitHub Actions runs. Establish that Actions are enabled and record results for the exact PR HEAD.
2. **BLOCKED / coverage:** no instrumented line/function/branch baseline is available yet. The new workflow produces reports but the numbers must come from a completed run.
3. **HIGH / installation safety:** platform-specific install, uninstall and self-update operations need isolated tests for permissions, partial/corrupt archives, rollback, and non-owned files.
4. **HIGH / untrusted metadata:** GitHub API and feed URLs were accepted based on a repository-wide prefix. The current PR restricts links to the advertised release tag and asset filename; confirm the tests and review compatibility with release naming.
5. **HIGH / native UI:** no confirmed true end-to-end installation/open/uninstall scenario on Linux/Windows/macOS, and no confirmed desktop UI smoke on real binaries.
6. **MEDIUM / browser:** WASM compile and bundle are not a substitute for browser E2E, responsive viewport or error-console checks.
7. **MEDIUM / input fuzzing:** malicious and truncated release-feed, checksum list, archive, and cache inputs need bounded fuzzing/property checks, preferably with minimized regression corpus.
8. **MEDIUM / platform coverage:** Ubuntu/Windows PR workflows do not provide macOS PR testing; release workflow checks are not equivalent to routine PR testing.

Do not merge a branch while required tests are failing, pending, or unavailable. Keep all cross-fork upstream PRs disallowed, and never make production install/update calls on live machines merely to exercise CI.
