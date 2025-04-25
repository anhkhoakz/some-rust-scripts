<div style="text-align: center">

<p><img src="https://i.imgur.com/N0ad6Az.png" width="200" alt="Project Icon"></p>

# Search Libraries
### An application to search for libraries

</div>

[![Rust](https://img.shields.io/github/actions/workflow/status/anhkhoakz/search-libraries/rust.yaml?branch=main&style=for-the-badge&logo=rust&color=%23dea584)](https://github.com/anhkhoakz/search-libraries/actions/workflows/rust.yaml)
![GitHub License](https://img.shields.io/github/license/anhkhoakz/search-libraries?style=for-the-badge)


## Installation

To use the libraries in this repository, ensure you have [Rust](https://www.rust-lang.org/) installed. Clone the repository and build the libraries using Cargo:

```bash
# Clone the repository
git clone https://github.com/anhkhoakz/search-libraries.git

# Navigate to the project directory
cd search-libraries

# Build the project
cargo build
```

## Usage

Import the desired library into your Rust project. For example:

```rust
use search_libraries::example_library;

fn main() {
    // Example usage
    example_library::search("query");
}
```

Or you can run directly:

```bash
# search-libraries repo package
search-libraries npm express
search-libraries docker node
```

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Commit your changes and push them to your fork.
4. Submit a pull request with a detailed description of your changes.

## License

This project is licensed under the GPLv2 License. See the [LICENSE](LICENSE) file for details.
