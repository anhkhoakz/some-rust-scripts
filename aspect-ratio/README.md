# Aspect Ratio

## Table of Contents

- [Aspect Ratio](#aspect-ratio)
  - [Table of Contents](#table-of-contents)
  - [About](#about)
  - [Getting Started](#getting-started)
    - [Prerequisites](#prerequisites)
    - [Building](#building)
    - [Installing](#installing)
    - [Uninstalling](#uninstalling)
  - [Usage](#usage)
  - [Examples](#examples)
  - [License](#license)

## About

**Aspect Ratio** is a simple CLI tool written in Rust for quickly reducing width and height values to their simplest aspect ratio form. It supports multiple input formats and is designed for fast, efficient use in the terminal.

## Getting Started

Follow these instructions to build and use the project on your local machine.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.82.0 or later)
- Cargo (comes with Rust)

### Building

Clone the repository and build the project:

```sh
git clone https://github.com/anhkhoakz/some-rust-scripts/
cd aspect-ratio
cargo build --release
```

### Installing

To install the binary system-wide (requires sudo):

```sh
sudo make install
```

You can also install it from crates.io:

```sh
cargo install aspect-ratio-cli
```

### Uninstalling

To remove the installed binary:

```sh
make uninstall
```

Or, if installed via Cargo:

```sh
cargo uninstall aspect-ratio-cli
```

## Usage

You can run the tool with:

```sh
./target/release/aspect-ratio-cli <width> <height>
./target/release/aspect-ratio-cli <width>x<height>
./target/release/aspect-ratio-cli <width>:<height>
```

Or, if installed system-wide or via Cargo, you can run it directly:

```sh
aspect-ratio-cli <width> <height>
aspect-ratio-cli <width>x<height>
aspect-ratio-cli <width>:<height>
```

## Examples

```sh
$ aspect-ratio-cli 1920 1080
16:9

$ aspect-ratio-cli 1280x720
16:9

$ aspect-ratio-cli 1024:768
4:3
```

If you provide invalid input, the tool will print an error and usage instructions.

## License

This project is licensed under the GNU General Public License version 2. See the [LICENSE](LICENSE) file for details.
