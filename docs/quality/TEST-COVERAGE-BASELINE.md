# Test and coverage baseline — 2026-10-12

## Reference

- Fork: `Rwanbt/craft-launcher`.
- Base: `main` at `1aaf7b27be2dba970d1a77f96aef6aad52352730` (identical to `storytold/craft-launcher/main` at reconnaissance time).
- Work: `test/deterministic-download-contracts`.
- Workspace: six Rust packages — `crates/core`, `crates/native`, `crates/ui-egui`, two applications in `apps/`, and `xtask`.
- Native desktop (Linux, Windows, macOS/FreeBSD) and WASM browser have different testing needs.

## Evidence and baseline status

- **Instrumented coverage on base:** **NOT MEASURED**. No line/function/branch percentages may be asserted.
- **Test runner collection on base:** **NOT EXECUTED**. Counting `#[test]` declarations is not equivalent to runner collection.
- **Existing workflow definitions:** `.github/workflows/ci.yml` has Ubuntu/Windows format, Clippy, tests, and WASM build checks. Other workflow files cover packaging, FreeBSD, Windows ARM64, and releases.
- **Existing fork Actions executions at reconnaissance:** **0 reported**; this is not a passed or failed run.
- **Local execution:** unavailable in this session (no Rust toolchain in runtime and GitHub host resolution unavailable).
- **New measurement job:** `.github/workflows/quality-coverage.yml` runs instrumented workspace tests with `cargo-llvm-cov` on Ubuntu and uploads LCOV and JSON reports. No coverage threshold is imposed until the first valid report is inspected.

## How to obtain the first trustworthy metric

1. On the exact proposed commit, inspect the `Rust coverage` workflow run and its test results.
2. Download `rust-coverage-<SHA>` and inspect `coverage.json` and `coverage.lcov`.
3. Record lines, functions and regions/branches for each crate, and identify compiler/configuration exclusions.
4. Measure additional compatible features and targets separately rather than adding incompatible percentages.
5. Keep UI native smoke/E2E and real installation tests distinct from Rust line coverage.

No file is declared 100% covered without a report for the matching commit and target.
