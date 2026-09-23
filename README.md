# space-crab

no alpha release yet

# Disk Space Analyzer

A fast, cross-platform command-line tool written in Rust to analyze disk usage.

## Ci
![CI](https://github.com/serhii-pokrovskyi/space-crab/actions/workflows/ci-main.yml/badge.svg?branch=main)

## Features

- Recursive directory scanning
- File size aggregation
- Sorted output
- Top N results display
- Human-readable sizes
- Command-line interface

## Releasing

`spacecrab-core` and `spacecrab` are always released together with the same version.

1. In the root `Cargo.toml`, set the new version in both `[workspace.package] version` and `[workspace.dependencies] spacecrab-core` (`version = "=X.Y.Z"`).
2. Run `cargo test --workspace` (updates `Cargo.lock`), commit, and merge to `main`.
3. Create a GitHub release with tag `vX.Y.Z`. The `Release` workflow publishes `spacecrab-core`, then `spacecrab`.

If `spacecrab-core` was published but the `spacecrab` job failed, use **Re-run failed jobs** on that run. Don't use "Re-run all jobs" and don't re-create the release: crates.io rejects a second upload of the same `spacecrab-core` version, so the CLI would be skipped again.

## License

This project is dual-licensed under either:

- [MIT License](LICENSE-MIT.MD)
- [Apache License, Version 2.0](LICENSE-APACHE.MD)

at your option.
