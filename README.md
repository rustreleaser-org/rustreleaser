<p align="center">
    <img src="assets/header.png" alt="rustreleaser-header" />
</p>

[![Crates.io](https://img.shields.io/crates/v/rustreleaser)](https://crates.io/crates/rustreleaser)
[![GitHub](https://img.shields.io/github/license/cestef/rustreleaser)](LICENSE)
[![Release](https://img.shields.io/github/v/release/cestef/rustreleaser)](https://github.com/cestef/rustreleaser/releases/latest)
[![Homebrew](https://img.shields.io/homebrew/v/rustreleaser)](https://formulae.brew.sh/formula/rustreleaser)

Release automation for Rust projects. The missing [`goreleaser`](https://goreleaser.com) for Rust.

## Features

- [ ] Platforms support
  - [x] Linux
  - [x] MacOS
  - [ ] Windows
- [x] Building via `cargo` and `cross`
- [x] Publishing
  - [x] [GitHub](https://github.com)
  - [x] [Homebrew](https://brew.sh)
  - [x] [crates.io](https://crates.io)
  - [ ] [Snapcraft](https://snapcraft.io)
  - [ ] [winget](https://winget.run) 
  - [ ] [DockerHub](https://hub.docker.com)
  - [ ] HTTP upload
  - [ ] [S3](https://aws.amazon.com/s3)
- [ ] Changelog generation

## Installation

```bash
cargo install rustreleaser
```

## Usage

```ansi
A tool to easily release Rust projects to GitHub, Homebrew, and crates.io

Usage: rr [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to the project [default: .]

Options:
  -c, --config <CONFIG>  Path to the config file [default: releaser.toml]
  -d, --dry-run          Dry run (do not upload anything)
  -o, --output <OUTPUT>  Output directory for temporary files [default: .]
  -h, --help             Print help
  -V, --version          Print version
```
