# space-crab

![CI](https://github.com/serhii-pokrovskyi/space-crab/actions/workflows/ci-main.yml/badge.svg?branch=main)

`spacecrab` shows what's taking up space on your disk. Point it at a folder and it lists every file in it with its size, then the total.

It's early, so the output is still simple. Folder totals and sorting are next, see the [roadmap](ROADMAP.md).

## Install

Grab the archive for your system from the [releases page](https://github.com/serhii-pokrovskyi/space-crab/releases), unpack it and put `spacecrab` somewhere on your `PATH`. There are builds for Linux, macOS and Windows, both x86-64 and ARM64.

The macOS binaries aren't signed, so macOS may refuse to run one you downloaded in a browser. Allow it in System Settings → Privacy & Security, or run:

```bash
xattr -d com.apple.quarantine spacecrab
```

If you use [cargo-binstall](https://github.com/cargo-bins/cargo-binstall), it fetches the same prebuilt binary:

```bash
cargo binstall spacecrab
```

Or build it from source (needs Rust 1.85 or newer):

```bash
cargo install spacecrab
```

## Usage

```bash
spacecrab ~/Downloads
```

Leave out the path to scan the current directory. Running it on this repo's `core/src` gives:

```text
core/src/path_scanner.rs 4.52 KiB
core/src/lib.rs 141 B
core/src/formatter.rs 2.01 KiB
core/src/calculator.rs 1.10 KiB

total size: 7.77 KiB
```

Sizes are file lengths, not the space the files actually take on disk.

The scanning code is its own crate, [`spacecrab-core`](https://crates.io/crates/spacecrab-core), if you want to use it from Rust.

## License

[MIT](LICENSE-MIT.MD) or [Apache-2.0](LICENSE-APACHE.MD), at your option.
