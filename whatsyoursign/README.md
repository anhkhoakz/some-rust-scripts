# whatsyoursign

[![Crates.io Version](https://img.shields.io/crates/v/whatsyoursign?style=for-the-badge)](https://crates.io/crates/whatsyoursign)
[![Crates.io Downloads](https://img.shields.io/crates/d/whatsyoursign?style=for-the-badge)](https://crates.io/crates/whatsyoursign)
[![Crates.io Size](https://img.shields.io/crates/size/whatsyoursign?style=for-the-badge)](https://crates.io/crates/whatsyoursign)

A macOS command-line tool to inspect code signatures of applications and executables.

## Overview

`whatsyoursign` provides detailed information about the code signature of macOS applications and executables, including:

- Signature validity and notarization status
- Signer type (Apple, Apple Developer ID, etc.)
- Signing authorities
- File hashes (MD5, SHA1, SHA256, SHA512, Code Directory Hash)
- Entitlements

## Prerequisites

- **macOS** (this tool only works on macOS)
- **Xcode Command Line Tools** (provides `codesign` and `spctl`)

To install Xcode Command Line Tools:

```sh
xcode-select --install
```

## Installation

### Build from source

```sh
git clone https://github.com/anhkhoakz/some-rust-scripts.git
cd some-rust-scripts/whatsyoursign
cargo build --release
```

### Install system-wide

```sh
strip target/release/whatsyoursign # I have no idea what this mean @@
sudo install -m 755 target/release/whatsyoursign /usr/local/bin/whatsyoursign
```

## Usage

### Basic usage

```sh
whatsyoursign --path /path/to/application.app
whatsyoursign --path /path/to/executable
```

### Examples

```sh
# Inspect an application bundle
whatsyoursign --path /Applications/Clop.app

# Inspect an executable
whatsyoursign --path /opt/homebrew/bin/tldr

# Inspect the current directory's executable
whatsyoursign --path ./target/release/whatsyoursign
```

## Output Format

The tool displays information in the following format:

```txt
Clop is validly signed & notarized (Signer: Apple Developer ID)
Clop
/Applications/Clop.app
Type: Application
Hashes:
  MD5:    E54E4C8931CFBA07C45A35454450D6B7
  SHA1:   C64D4053C1B30EC6E455B1183C40B599F2BC6EEA
  SHA256: 894E331AC2A5491E97178436A59B847697A64A01C80814C273121E9B3390F11E
  SHA512: 6C1C41F27C1C63F70479BED67136C9F64E00F23FE97DA7A189408F74B239A456C5849A1D1EF89E6311F81158AE5D32661DC1BC4A36E75245AF0BCCD23F813B58
  Code Directory Hash (SHA-256): 63E896E46D932FFA267B829264EF368984996E6354D0E339ED77930DC46D6523
Entitlements:
{
  "com.apple.application-identifier": "RDDXV84A73.com.lowtechguys.Clop",
  "com.apple.developer.icloud-container-identifiers": "RDDXV84A73",
  "com.apple.developer.ubiquity-kvstore-identifier": "RDDXV84A73.com.lowtechguys.Clop",
  "com.apple.security.automation.apple-events": true,
  "com.apple.security.temporary-exception.mach-lookup.global-name": "com.lowtechguys.Clop.optimisationServiceResponse",
  "com.apple.security.temporary-exception.mach-register.global-name": "com.lowtechguys.Clop.optimisationService"
}

Sign Auths: › Developer ID Application: Alin Panaitiu (RDDXV84A73)
            › Developer ID Certification Authority
            › Apple Root CA
```

### Output Fields

- **Status line**: Shows whether the signature is valid and/or notarized, along with the signer type
- **Name**: Extracted from the code signature identifier
- **Path**: The path to the inspected file
- **Type**: Application bundle, Executable, or Unknown
- **Hashes**: MD5, SHA1, SHA256, SHA512, and Code Directory Hash (SHA-256)
- **Entitlements**: Formatted entitlements plist (if present)
- **Sign Auths**: List of signing certificate authorities

## CLI Reference

```sh
whatsyoursign [OPTIONS] --path <PATH>
```

| Flag | Description |
| --- | --- |
| `-p, --path <PATH>` | Path to the application bundle or executable to inspect (required) |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

## How It Works

`whatsyoursign` uses macOS's built-in code signing tools:

1. **`codesign -dvvv`**: Extracts signature information, format, authorities, and code directory hash
2. **`spctl -a -v`**: Validates the signature and checks notarization status
3. **`md5` and `shasum`**: Calculates file hashes
4. **`codesign -d --entitlements`**: Extracts entitlements plist

## Exit Codes

- **0**: Success - signature inspection completed
- **1**: Failure - error occurred (file not found, missing dependencies, etc.)

## Error Handling

The tool will exit with an error if:

- The specified path does not exist
- `codesign` or `spctl` are not found in PATH
- The file cannot be inspected (not a signed binary/app)
- Any system command fails

## Examples of Output

### Validly signed and notarized application

```txt
Clop is validly signed & notarized (Signer: Apple Developer ID)
Clop
/Applications/Clop.app
Type: Application
Hashes:
  MD5:    E54E4C8931CFBA07C45A35454450D6B7
  SHA1:   C64D4053C1B30EC6E455B1183C40B599F2BC6EEA
  SHA256: 894E331AC2A5491E97178436A59B847697A64A01C80814C273121E9B3390F11E
  SHA512: 6C1C41F27C1C63F70479BED67136C9F64E00F23FE97DA7A189408F74B239A456C5849A1D1EF89E6311F81158AE5D32661DC1BC4A36E75245AF0BCCD23F813B58
  Code Directory Hash (SHA-256): 63E896E46D932FFA267B829264EF368984996E6354D0E339ED77930DC46D6523
Entitlements:
{
  "com.apple.application-identifier": "RDDXV84A73.com.lowtechguys.Clop",
  "com.apple.developer.icloud-container-identifiers": "RDDXV84A73",
  "com.apple.developer.ubiquity-kvstore-identifier": "RDDXV84A73.com.lowtechguys.Clop",
  "com.apple.security.automation.apple-events": true,
  "com.apple.security.temporary-exception.mach-lookup.global-name": "com.lowtechguys.Clop.optimisationServiceResponse",
  "com.apple.security.temporary-exception.mach-register.global-name": "com.lowtechguys.Clop.optimisationService"
}

Sign Auths: › Developer ID Application: Alin Panaitiu (RDDXV84A73)
            › Developer ID Certification Authority
            › Apple Root CA
```

### Unsigned or invalid signature

```txt
whatsyoursign is not validly signed
whatsyoursign
./target/release/whatsyoursign
Type: Executable
Hashes:
  MD5:    FD61EB11F4B78D2BF817C165C08CAF72
  SHA1:   93A350BFCEB65A13F61E3C67A26D9BEBC860BF6B
  SHA256: 9BAE775EAAE3E1DD5930E02BE4AFF6F94BD61BAA3DC300B0400344A911E98C2E
  SHA512: 42CCC5A49261F49B1A2E633334CA28DDB7AFB3599FE3179247889FF45589D0C0BC67DCDFDE75A34B3D42AFC9F91176B353CE246A75D71713DE7B4BCA5D29A327
  Code Directory Hash (SHA-256): 5715DED9612705B9B221B09A9077F7F46981509E903B18169EEF21B28B776926
Entitled: View Entitlements
Sign Auths: (none)
```

## Contributing

Bug reports and pull requests are welcome. Please open an issue first if you would like to propose a sizable change. Make sure that `cargo fmt`, `cargo clippy` pass before submitting.

## License

Distributed under the AGPL-3.0 license. See [LICENSE](LICENSE) for details.
