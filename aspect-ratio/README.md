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
git clone <repo-url>
cd aspect-ratio
cargo build --release
```

### Installing

To install the binary system-wide (requires sudo):

```sh
make install
```

### Uninstalling

To remove the installed binary:

```sh
make uninstall
```

## Usage

You can run the tool with:

```sh
./target/release/aspect-ratio <width> <height>
./target/release/aspect-ratio <width>x<height>
./target/release/aspect-ratio <width>:<height>
```

Or, if installed:

```sh
aspect-ratio <width> <height>
aspect-ratio <width>x<height>
aspect-ratio <width>:<height>
```

## Examples

```sh
$ aspect-ratio 1920 1080
16:9

$ aspect-ratio 1280x720
16:9

$ aspect-ratio 1024:768
4:3
```

If you provide invalid input, the tool will print an error and usage instructions.

## License

This project is licensed under the GNU General Public License version 2. See the [LICENSE](LICENSE) file for details.
