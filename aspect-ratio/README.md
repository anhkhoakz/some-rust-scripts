# aspect-ratio-cli

[![Crates.io Version](https://img.shields.io/crates/v/aspect-ratio-cli?style=for-the-badge)](https://crates.io/crates/aspect-ratio-cli)
[![Crates.io Downloads](https://img.shields.io/crates/d/aspect-ratio-cli?style=for-the-badge)](https://crates.io/crates/aspect-ratio-cli)
[![Crates.io Size](https://img.shields.io/crates/size/aspect-ratio?style=for-the-badge)](https://crates.io/crates/aspect-ratio-cli)
[![License: GPL-2.0](https://img.shields.io/crates/l/aspect-ratio-cli?style=for-the-badge&logo=gnu&color=A42E2B)](LICENSE)

## About

**aspect-ratio-cli** is a fast, simple CLI tool written in Rust for reducing width and height values to their simplest aspect ratio form. It supports multiple input formats and is designed for efficient use in the terminal.

## Features

- Reduce width and height to the simplest aspect ratio (e.g., 1920x1080 → 16:9)
- Supports input as `<width> <height>`, `<width>x<height>`, or `<width>:<height>`
- Convert aspect ratios to a target width or height
- Show decimal representation of aspect ratios
- Generate shell completions for popular shells

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.86.0 or later)
- Cargo (comes with Rust)
- [Just](https://github.com/casey/just) (optional, for build and install from source)

### Building

```sh
git clone https://github.com/anhkhoakz/some-rust-scripts.git
cd some-rust-scripts/aspect-ratio
just build
```

### Installing

System-wide (optional, requires sudo):

```sh
just install
```

Or from crates.io:

```sh
cargo install aspect-ratio-cli
```

### Uninstalling

```sh
just uninstall
# or, if installed via Cargo:
cargo uninstall aspect-ratio-cli
```

## Usage

```sh
aspect-ratio-cli [COMMAND]
```

### Available Commands

- `convert`: Convert an aspect ratio to a target width or height.
- `info`: Show info about an aspect ratio.
- `completions`: Generate shell completions for popular shells.
- `calc`: Reduce an aspect ratio to its simplest form.
- `-h, --help`: Print help for the tool or command.
- `-V, --version`: Print the version of the tool.

### Example

```sh
aspect-ratio-cli calc 1920 1080
```

If you provide invalid input, the tool will print an error and usage instructions.

## Contributing

Contributions are welcome! Please open issues or pull requests on GitHub. See the root `CONTRIBUTING.md` for guidelines.

## License

This project is licensed under the GNU General Public License v2.0. See the [LICENSE](LICENSE) file for details.
