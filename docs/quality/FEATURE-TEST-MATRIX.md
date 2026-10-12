# Feature-to-test traceability (initial risk-first inventory)

Statuses refer to **actual execution evidence**, not merely to the presence of source tests. This is an initial map and must be extended by code/API discovery and CI artifacts.

| ID | Feature / package | Behavioural contracts & risks | Available evidence | Current status |
| --- | --- | --- | --- | --- |
| CL-001 | GitHub release parsing (`core/release.rs`) | Valid JSON, untrusted URLs, release versions, SHA-256 digest, wrong tag/file | Existing unit tests; added exact-release asset URL regression test | PARTIAL — new SHA not executed |
| CL-002 | API feed (`core/feed.rs`) | Freshness, repo renames, URL origin, wrong tag/file, mirror opt-in | Existing unit tests; added feed cross-release URL regression test | PARTIAL — new SHA not executed |
| CL-003 | Platform/asset selection (`core/platform.rs`) | Native first, fallback correctness, OS/architecture isolation | In-source unit tests | PARTIAL — no new SHA execution |
| CL-004 | Versions and releases (`core/version.rs`) | Stable/prerelease comparisons, package-manager normalization | In-source unit tests | PARTIAL — no new SHA execution |
| CL-005 | HTTPS download/verify (`native/http.rs`) | Resume, checksums, redirect, network failures, cancellation | In-source tests; integration/E2E pending classification | PARTIAL |
| CL-006 | Install/remove/self-update (`native/install.rs`, `self_update.rs`) | Avoid deleting unrelated files, atomicity, permissions, recovery | Needs isolated OS-specific scenarios and packaging tests | UNVERIFIED |
| CL-007 | Manifest and app detection (`native/manifest.rs`, `detect.rs`) | Persist/reload, corrupt files, installed versions, paths | Manifest round-trip test exists; end-to-end inventory unverified | PARTIAL |
| CL-008 | Desktop UI, tray and notifications (`ui-egui`, `apps/artcraft-launcher`) | Launch, settings, progress, pause/cancel, keyboard actions, responsiveness | egui tests present; real executable E2E not evidenced | UNVERIFIED |
| CL-009 | WebAssembly catalog (`apps/artcraft-launcher-web`) | Browser startup, asset links, mobile viewport, JS/browser errors | WASM check + trunk build configured; browser E2E not evidenced | UNVERIFIED |
| CL-010 | Localization and accessibility (`ui-egui/src/i18n`) | Locale fallback and complete menus | Unit/integration test sources present; CI outcome pending | PARTIAL |
| CL-011 | Multiplatform packaging | Installer creation, smoke, installation/removal and file integrity | Workflows configured for packaging/release; real artifacts unverified | UNVERIFIED |

**Verification rule:** move a row to VERIFIED only with passing behaviour-specific tests plus the exact commit SHA and applicable CI job/artefact. Coverage percentages alone never prove these contracts.
