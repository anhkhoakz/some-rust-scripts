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

## About

Count Characters is a simple CLI tool written in Rust that counts the number of characters in a given input, including support for reading from files. It trims leading and trailing blank lines and is designed for quick, efficient use in the terminal.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.82.0 or later)
- Cargo (comes with Rust)

### Installing

Clone the repository:

```sh
git clone <repo-url>
cd count-characters
```

Build the project:

```sh
make build
```

Install the binary (optional, requires sudo):

```sh
make install
```

To uninstall:

```sh
make uninstall
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
