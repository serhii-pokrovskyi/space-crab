# space-crab

![CI](https://github.com/serhii-pokrovskyi/space-crab/actions/workflows/ci-main.yml/badge.svg?branch=main)

`spacecrab` shows what's taking up space on your disk. Point it at a folder and it lists everything in it, each with its total size on disk, smallest first, with the total for the whole folder last.

It's early: until 1.0 a release may still change the output. See the [roadmap](ROADMAP.md).

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

Leave out the path to scan the current directory. On a Mac, running it on this repo's `core` folder gives something like:

```text
   4.00 KiB	core/Cargo.toml
   4.00 KiB	core/LICENSE-MIT.MD
  12.00 KiB	core/LICENSE-APACHE.MD
  28.00 KiB	core/src/
  48.00 KiB	core
```

Each line is a size, a tab, then the path. Folders inside end with a slash (`\` on Windows), and the last line is the folder you gave, as you typed it. Nothing is printed until the scan is done, since the list is sorted.

Sizes are the space files take on disk, as `du` counts it, and a file with several hard links counts once. `-A` counts file lengths instead. `-b` prints exact byte counts for scripts, still space on disk unless you add `-A`. On Windows sizes are always file lengths, and every hard link counts.

`-x` stays on the filesystem the folder is on, like `du -x`; it does nothing on Windows. On a Mac, `-x /` still goes into the data volume and counts it twice, so to measure your data, scan `-x /System/Volumes/Data`.

The scanning code is its own crate, [`spacecrab-core`](https://crates.io/crates/spacecrab-core), if you want to use it from Rust.

## License

[MIT](LICENSE-MIT.MD) or [Apache-2.0](LICENSE-APACHE.MD), at your option.
