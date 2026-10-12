# Test and coverage baseline — 2026-10-12

## Exact references and limits

- Fork: `Rwanbt/craft-launcher`.
- Original `main`: `1aaf7b27be2dba970d1a77f96aef6aad52352730` (matched upstream when inspected). **Original main itself was not instrumented.**
- Measured PR commit: `cb2429d4a68c13890954ca60ec00736e02c8c3f0` on `test/deterministic-download-contracts`.
- GitHub Actions coverage run: https://github.com/Rwanbt/craft-launcher/actions/runs/38273175797
- Artifact: https://github.com/Rwanbt/craft-launcher/actions/runs/38273175797/artifacts/11758590002 (`coverage.lcov`, `coverage.json`).
- Workspace: six Rust packages — core, native, ui-egui, desktop app, web app, xtask.
- Target: Ubuntu GitHub-hosted Linux runner, `cargo llvm-cov --workspace --locked --lcov --output-path coverage.lcov`, then `cargo llvm-cov report --summary-only`.
- This is **workspace source coverage under one Linux test configuration**, not a release-build, all-features, WASM/browser, FreeBSD, Windows or macOS end-to-end coverage measure.

## Measured instrumented baseline

| Metric | Covered / total | Percent |
| --- | ---: | ---: |
| Lines | 8,862 / 11,723 | **75.59%** |
| Functions | 1,386 / 1,888 | **73.41%** |
| Regions | 20,588 / 27,062 | **76.08%** |
| Branches | Not reported (0 / 0) | Not measurable in this report |

- Instrumented test runner: **301 passed, 0 failed, 3 ignored**, across 16 reported test-runner summaries. These are executable tests, not source-file counts.
- `cargo-llvm-cov` also warned that six functions had mismatched profile data. The report is retained as a measured baseline, but this warning needs investigation before relying on fine-grained comparisons.
- Exact CI run also completed the new coverage job successfully. Corresponding workflow `CI` (Linux, Windows, WASM/trunk) and packaging lint are green for the same measured commit.

## Selected measured files (Linux)

| File | Lines covered | Interpretation |
| --- | ---: | --- |
| `crates/core/src/release.rs` | 93.15% | release parsing tested, edge paths remain |
| `crates/core/src/feed.rs` | 92.93% | feed parsing tested |
| `crates/native/src/http.rs` | 87.39% | downloader unit and local HTTP tests |
| `crates/native/src/source.rs` | **0.00%** | release source/cache tests missing at this exact commit |
| `crates/native/src/backend.rs` | 58.47% | native integration gaps |
| `crates/native/src/install.rs` | 67.70% | OS-specific install/remove gaps |
| `crates/native/src/self_update.rs` | 62.78% | update recovery tests required |
| `crates/native/src/login.rs` | **0.00%** | integration coverage absent |
| `crates/ui-egui/src/views.rs` | 89.71% | UI behaviour tested, not a real binary E2E |
| `apps/artcraft-launcher/src/tray.rs` | **8.82%** | OS/tray integration not verified |
| `xtask/src/local.rs` | 35.42% | developer local packaging integration gaps |

Separate draft PRs #2, #3 and #4 work on native writes, cached source/throttling and tree inventory respectively. Their changes are **not included** in the measurement above. Recompute after integration instead of adding percentages from different commits.

## Next gates

1. Establish a clean, warning-investigated profile for the exact combined revision after approved integration.
2. Add deterministic tests and/or dependency-injection seams for native source, backend, installation, self-update and tray behaviour.
3. Execute actual desktop smoke and browser E2E in isolated environments; do not substitute unit/UI-harness tests for those.
4. Collect feature/platform-specific results separately and document all legitimate exclusions.
5. Set an anti-regression threshold only after agreeing on stable measurement and exclusions; never claim 100% from a single line-coverage percentage.
