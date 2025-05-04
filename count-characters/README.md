# Count Characters

## Table of Contents

- [Count Characters](#count-characters)
  - [Table of Contents](#table-of-contents)
  - [About](#about)
  - [Getting Started](#getting-started)
    - [Prerequisites](#prerequisites)
    - [Installing](#installing)
  - [Usage](#usage)
  - [License](#license)

![Crates.io Version](https://img.shields.io/crates/v/count-characters?style=for-the-badge)
![Crates.io Total Downloads](https://img.shields.io/crates/d/count-characters?style=for-the-badge)
![Crates.io Size (version)](https://img.shields.io/crates/size/count-characters/0.1.3?style=for-the-badge)

## About

Count Characters is a simple CLI tool written in Rust that counts the number of characters in a given input, including support for reading from files. It trims leading and trailing blank lines and is designed for quick, efficient use in the terminal.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.86.0 or later)
- Cargo (comes with Rust)

### Installing

Clone the repository:

```sh
git clone https://github.com/anhkhoakz/some-rust-scripts/
cd some-rust-scripts
cd count-characters
```

Install the binary (optional, requires sudo):

```sh
just install
# or from crates.io:
cargo install count-characters
```

To uninstall:

```sh
just uninstall
# or from crates.io:
cargo uninstall count-characters
```

## Usage

You can run the tool with:

```sh
./target/release/count-characters
```

Or, if installed:

```sh
count-characters
```

Paste your text, then press `Ctrl-D` (on Mac/Linux) or `Ctrl-Z` (on Windows) to finish input. The tool will output the number of characters in your input (excluding leading/trailing blank lines).

**Example:**

```sh
$ count-characters
Paste your text, then press Ctrl-D (on Mac/Linux) or Ctrl-Z (on Windows) to finish:
Hello, world!

Input contains 13 characters.
```

```sh
$ count-characters /path/to/file.txt
Input contains 42 characters.
```

## License

This project is licensed under the GNU General Public License version 2 - see the [LICENSE](LICENSE) file for details.
