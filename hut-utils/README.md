# hut-utils

[![Crates.io Version](https://img.shields.io/crates/v/hut-utils?style=for-the-badge)](https://crates.io/crates/hut-utils)
[![Crates.io Downloads](https://img.shields.io/crates/d/hut-utils?style=for-the-badge)](https://crates.io/crates/hut-utils)
[![Crates.io Size](https://img.shields.io/crates/size/aspect-ratio?style=for-the-badge)](https://crates.io/crates/hut-utils)
[![License](https://img.shields.io/crates/l/hut-utils?style=for-the-badge&logo=gnu&color=A42E2B)](LICENSE)

## About

**hut-utils** is a fast, simple CLI extension written in Rust for interacting with sourcehut. It supports multiple input formats and is designed for efficient use in the terminal.

## Features

- Update a paste on sourcehut by deleting the old one and creating a new one.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.87.0 or later)
- Cargo (comes with Rust)
- [Just](https://github.com/casey/just) (optional, for build and install from source)
- [Sourcehut](https://sourcehut.org/) account
- [Sourcehut CLI](https://sr.ht/~xenrox/hut/)

### Building

```sh
git clone https://git.sr.ht/~anhkhoakz/hut-cli-utils
cd hut-cli-utils
just build
```

### Installing

System-wide (optional, requires sudo):

```sh
just install
```

Or from crates.io:

```sh
cargo install hut-utils
```

### Uninstalling

```sh
just uninstall
# or, if installed via Cargo:
cargo uninstall hut-utils
```

## Usage

```sh
hut-utils [COMMAND]
```

### Available Commands

- `paste edit`: Edit a paste on sourcehut by deleting the old one and creating a new one.
- `paste rename`: Rename an existing paste on sourcehut.
- `-h, --help`: Print help for the tool or command.
- `-V, --version`: Print the version of the tool.

### Example

```sh
hut-utils paste edit --source-file <source-file> --remote-file <remote-file> --visibility <visibility>
hut-utils paste rename --current-name <current-name> --new-name <new-name>
```

If you provide invalid input, the tool will print an error and usage instructions.

## Contributing

Contributions are welcome! Please open issues or pull requests on [Sourcehut](https://git.sr.ht/~anhkhoakz/hut-cli-utils). See the root `CONTRIBUTING.md` for guidelines.

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.
