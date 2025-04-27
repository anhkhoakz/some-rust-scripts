# JSON Compare CLI

## Table of Contents

- [JSON Compare CLI](#json-compare-cli)
  - [Table of Contents](#table-of-contents)
  - [About](#about)
  - [Getting Started](#getting-started)
    - [Prerequisites](#prerequisites)
    - [Building](#building)
    - [Installing](#installing)
    - [Uninstalling](#uninstalling)
  - [Usage](#usage)
  - [License](#license)

![Crates.io Version](https://img.shields.io/crates/v/json-compare-cli?style=for-the-badge)
![Crates.io Total Downloads](https://img.shields.io/crates/d/json-compare-cli?style=for-the-badge)
![Crates.io Size (version)](https://img.shields.io/crates/size/json-compare-cli/0.1.0?style=for-the-badge)

## About

**JSON Compare CLI** is a simple CLI tool written in Rust for comparing JSON files and printing the differences in a human-readable format. It supports multiple input formats and is designed for fast, efficient use in the terminal.

## Getting Started

Follow these instructions to build and use the project on your local machine.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.82.0 or later)
- Cargo (comes with Rust)

### Building

Clone the repository and build the project:

```sh
git clone https://github.com/anhkhoakz/some-rust-scripts/
cd json-compare-cli
cargo build --release
```

### Installing

To install the binary system-wide (requires sudo):

```sh
sudo make install
```

You can also install it from crates.io:

```sh
cargo install json-compare-cli
```

### Uninstalling

To remove the installed binary:

```sh
make uninstall
```

Or, if installed via Cargo:

```sh
cargo uninstall json-compare-cli
```

## Usage

You can run the tool with:

```sh
cargo run --release -- file1.json file2.json

```

Or, if installed system-wide or via Cargo, you can run it directly:

```sh
json-compare-cli file1.json file2.json
```

If you provide invalid input, the tool will print an error and usage instructions.

## License

This project is licensed under the GNU General Public License version 2. See the [LICENSE](LICENSE) file for details.
