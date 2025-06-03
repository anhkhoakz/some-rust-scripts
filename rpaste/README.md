# rpaste

A command-line tool for pasting text to [snip.dssr.ch](https://snip.dssr.ch/), a PrivateBin instance.

## Features

- Password protection for pastes
- File attachments
- Automatic syntax highlighting
- Markdown support
- Burn after reading
- Open discussion
- Client-side encryption

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Paste from stdin
echo "Hello, world!" | rpaste

# Paste from a file
rpaste -f file.txt

# Create a password-protected paste
rpaste -p "mysecret" -f file.txt

# Set expiration time
rpaste -e 1day -f file.txt

# Use syntax highlighting
rpaste -s -f file.rs

# Parse as markdown
rpaste -m -f README.md

# Burn after reading
rpaste -b -f file.txt

# Allow discussion
rpaste -o -f file.txt

# Attach a file
rpaste -a image.png -f file.txt
```

## Command Line Options

- `-f, --file FILE`: Read from a file instead of stdin
- `-p, --password PASSWORD`: Create a password protected paste
- `-e, --expire EXPIRE`: Expiration time (5min, 10min, 1hour, 1day, 1week, 1month, 1year, never)
- `-s, --sourcecode`: Use source code highlighting
- `-m, --markdown`: Parse paste as markdown
- `-b, --burn`: Burn paste after reading
- `-o, --opendiscussion`: Allow discussion for the paste
- `-a, --attachment FILE`: Specify path to a file to attach

## Security

This tool uses client-side encryption to ensure the privacy of your pastes. The encryption is done using AES-256-GCM with a random passphrase. If a password is provided, it is combined with the passphrase for additional security.

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Copyright

Copyright (c) 2025 Nguyễn Huỳnh Anh Khoa
