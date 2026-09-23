# space-crab

![CI](https://github.com/serhii-pokrovskyi/space-crab/actions/workflows/ci-main.yml/badge.svg?branch=main)

A fast, cross-platform command-line tool written in Rust to analyze disk usage.

Early development: the CLI lists every file under the current directory with its size, followed by the total.

## Install

```bash
cargo install spacecrab
```

Requires Rust 1.85 or newer.

## Usage

Run it in the directory you want to analyze:

```bash
spacecrab
```

## Features

- Recursive directory scanning
- Per-file sizes and the total size
- Human-readable sizes (KiB, MiB, GiB, …)

Planned:

- Sorted output
- Top N results
- Command-line arguments (path to scan, options)

## Crates

- [`spacecrab`](https://crates.io/crates/spacecrab): the command-line tool
- [`spacecrab-core`](https://crates.io/crates/spacecrab-core): the scanning library it is built on

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
