# space-crab

![CI](https://github.com/serhii-pokrovskyi/space-crab/actions/workflows/ci-main.yml/badge.svg?branch=main)

`spacecrab` shows what's taking up space on your disk. Point it at a folder and it lists what's directly in it with the total size of each, biggest last, then the total for the whole folder.

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

```text
spacecrab [OPTIONS] [PATH]
```

Leave out the path to scan the current folder. Running it on this repo's `core` folder on a Mac gives:

```text
   4.00 KiB	core/Cargo.toml
   4.00 KiB	core/LICENSE-MIT.MD
  12.00 KiB	core/LICENSE-APACHE.MD
  36.00 KiB	core/src/
  56.00 KiB	core
```

Each line is the size, a tab, then the path, so it's easy to take apart in scripts. Folders end with a slash, and the last line is the total. Nothing shows up until the whole scan is done.

`-b` prints exact bytes, which is handier for scripts. Here's the total and the five biggest things in your home folder:

```bash
spacecrab -b ~ | sort -rn | head -6
```

To measure a whole disk, add `-x` so the scan stays on that one filesystem. On a Mac that's your data volume:

```bash
sudo spacecrab -x /System/Volumes/Data
```

Your terminal also needs Full Disk Access for that; `sudo` alone isn't enough. `spacecrab --help` lists all the options.

## What the sizes mean

It counts the space things actually take on disk, the same way `du` does. So a big sparse file that's mostly empty only counts what's really there, and a file with several hard links only counts once. Folders count their own few blocks too. Symlinks are left alone, unless the path you give is one. Add `-A` if you want file lengths instead. `-b` on its own still counts disk space, just in bytes.

On Windows it's always file lengths, every hard link counts, and `-x` does nothing.

If something can't be read, you'll see an error on stderr and the scan keeps going. Exit code 0 means everything got counted, 1 means something was missed or the output couldn't be written, and 2 means the command line was wrong. If the path itself can't be read, nothing is printed.

The scanning code is its own crate, [`spacecrab-core`](https://crates.io/crates/spacecrab-core), if you want to use it from Rust.

## License

[MIT](LICENSE-MIT.MD) or [Apache-2.0](LICENSE-APACHE.MD), at your option.
