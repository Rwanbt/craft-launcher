# Deterministic test execution

All commands below are **instructions**, not claims of successful execution.

## CI

- `.github/workflows/ci.yml`: Linux/Windows `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`; Linux WASM/trunk build.
- `.github/workflows/quality-coverage.yml`: Linux `cargo llvm-cov --workspace --locked --lcov --output-path coverage.lcov`, followed by JSON report and downloadable artifacts.
- Other workflows: packaging lint, FreeBSD, Windows ARM64 and release checks. Release work must not be run to validate a normal PR.

## Local reproduction on a suitable host

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo xtask wasm
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked
cargo llvm-cov --workspace --locked --lcov --output-path coverage.lcov
cargo llvm-cov report --json --output-path coverage.json
```

On Linux install the windowing development headers described in the repository README and CI workflow before building native crates. For a focused regression run:

```sh
cargo test -p artcraft-launcher-core release_assets_must_match_the_tag_and_name_exactly
cargo test -p artcraft-launcher-core feed_rejects_a_cross_tag_or_mismatched_asset_url
```

Tests of production installers or self-update should use disposable environments and synthetic signed fixtures as appropriate, never users' installations or chargeable services.

## Current execution register

| Reference | Tests | Coverage | CI |
| --- | --- | --- | --- |
| Baseline `main` `1aaf7b27be2dba970d1a77f96aef6aad52352730` | Not run in this session | Not measured | No runs reported by fork at audit |
| Draft PR head | Two security regression tests added | Await instrumented run | Await PR-triggered Actions |

When checks finish, update this record with the exact commit, workflow run link, job outcomes and measured report.
