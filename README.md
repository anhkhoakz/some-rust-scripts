# some-rust-scripts

## Table of Contents

- [some-rust-scripts](#some-rust-scripts)
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

**some-rust-scripts** is a collection of simple and useful CLI tools written in Rust. Each script solves a specific problem efficiently from the terminal. The tools support various input formats and are intended for fast, practical use.

## Getting Started

Follow these instructions to build and use the scripts on your local machine.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.86.0 or later)
- Cargo (comes with Rust)

### Building

Clone the repository and build the project:

```sh
git clone https://github.com/anhkhoakz/some-rust-scripts.git
cd some-rust-scripts
cd <script-name>
cargo build --release
```

### Installing

To install the binaries system-wide (requires sudo):

```sh
just install
```

### Uninstalling

To remove the installed binaries:

```sh
just uninstall
```

## Usage

Each script can be run from the `target/release` directory or, if installed, directly from your terminal. Example usage for a script:

```sh
./target/release/<script-name> [arguments]
```

Or, if installed:

```sh
<script-name> [arguments]
```

Refer to each script's help output for specific usage instructions.

## Examples

```sh
$ aspect-ratio 1920 1080
16:9

$ count-characters "Hello, world!"
13
```

If you provide invalid input, the tool will print an error and usage instructions.

## License

This project is licensed under the GNU General Public License version 2. See the [LICENSE](LICENSE) file for details.
