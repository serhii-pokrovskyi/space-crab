# space-crab

![CI](https://github.com/serhii-pokrovskyi/space-crab/actions/workflows/ci-main.yml/badge.svg?branch=main)

A fast, cross-platform command-line tool written in Rust to analyze disk usage.

Early development: the CLI lists every file under a directory with its size, followed by the total.

## Install

### Prebuilt binary

Download the archive for your platform from the [Releases page](https://github.com/serhii-pokrovskyi/space-crab/releases), unpack it, and put `spacecrab` somewhere on your `PATH`. Binaries are available for Linux (x86-64, ARM64), macOS (Apple Silicon, Intel) and Windows (x86-64, ARM64), each with a `.sha256` checksum file.

On macOS the binaries are not signed. If macOS blocks a file downloaded in a browser, allow it in **System Settings → Privacy & Security**, or run `xattr -d com.apple.quarantine spacecrab`.

### With cargo-binstall

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall), it downloads the prebuilt binary instead of compiling:

```bash
cargo binstall spacecrab
```

### From source

```bash
cargo install spacecrab
```

Requires Rust 1.85 or newer.

## Usage

```bash
spacecrab [PATH]
```

PATH is the directory to analyze. It defaults to the current directory.

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
