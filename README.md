<div align="center">

# 🔓 gemini-unlock

**Enable Chrome's built-in Gemini AI features outside the US**

> ⚡ A one-click script to bypass Chrome Gemini's region restrictions and unlock AI features for non-US users

[![Build Status](https://img.shields.io/github/actions/workflow/status/flyingzl/gemini-unlock/build.yml?branch=main)](https://github.com/flyingzl/gemini-unlock/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

[Features](#-features) • [Quick Start](#-quick-start) • [Installation](#-installation) • [Usage](#-usage) • [How it Works](#-how-it-works)

**Made by [@flyingzl](https://github.com/flyingzl)**

</div>

---

## ✨ Features

- 🚀 **One-Command Setup** - Enable Gemini features in seconds
- 🛡️ **Safe & Reversible** - Automatic backups, easy restore
- 🔒 **Type-Safe** - Built with Rust, uses serde_json for reliable parsing
- 🌍 **Cross-Platform** - Supports macOS, Linux, and Windows
- 📦 **Zero Dependencies** - Single binary, no runtime requirements
- 🧪 **Well-Tested** - 100% test coverage with 22+ tests
- 📝 **Clean Logs** - Structured logging for debugging
- ⚡ **Lightning Fast** - Optimized for minimal overhead

---

## 🎯 Quick Start

```bash
# Install (macOS/Linux)
cargo install gemini-unlock

# Run (Chrome must be closed first)
gemini-unlock

# Or automatically close Chrome, patch, and restart
gemini-unlock --kill-chrome
```

That's it! Restart Chrome and enjoy Gemini features. 🎉

---

## 📦 Installation

### Binary Release (Recommended)

Download the latest release for your platform from [Releases](https://github.com/flyingzl/gemini-unlock/releases):

```bash
# macOS (Apple Silicon)
curl -LO https://github.com/flyingzl/gemini-unlock/releases/latest/download/gemini-unlock-aarch64-apple-darwin.tar.gz
tar -xzf gemini-unlock-aarch64-apple-darwin.tar.gz
mv gemini-unlock /usr/local/bin/

# macOS (Intel)
curl -LO https://github.com/flyingzl/gemini-unlock/releases/latest/download/gemini-unlock-x86_64-apple-darwin.tar.gz
tar -xzf gemini-unlock-x86_64-apple-darwin.tar.gz
mv gemini-unlock /usr/local/bin/

# Linux
curl -LO https://github.com/flyingzl/gemini-unlock/releases/latest/download/gemini-unlock-x86_64-unknown-linux-gnu.tar.gz
tar -xzf gemini-unlock-x86_64-unknown-linux-gnu.tar.gz
mv gemini-unlock /usr/local/bin/

# Windows
# Download .exe from Releases page
```

### Cargo

```bash
cargo install gemini-unlock
```

### From Source

```bash
git clone https://github.com/flyingzl/gemini-unlock.git
cd gemini-unlock
cargo install --path .
```

---

## 💻 Usage

### Basic Usage

```bash
# Apply patches (Chrome must be closed)
gemini-unlock

# Automatically close Chrome before patching
gemini-unlock -k

# Restore from backup
gemini-unlock -r

# Show help
gemini-unlock --help

# Enable debug logging
RUST_LOG=debug gemini-unlock
```

### Command-Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--kill-chrome` | `-k` | Close running Chrome before applying patches |
| `--restore` | `-r` | Restore Local State from backup instead of patching |
| `--help` | `-h` | Print help information |
| `--version` | `-V` | Print version information |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG=info` | Enable info-level logging (default) |
| `RUST_LOG=debug` | Enable debug-level logging |
| `RUST_LOG=warn` | Enable only warnings |
| `RUST_LOG=error` | Enable only errors |

---

## 🔧 How it Works

This tool modifies Chrome's Local State configuration file to enable Gemini features by:

1. **Detecting** your operating system and Chrome config location
2. **Backing up** your original Local State file (as `Local State.bak`)
3. **Patching** specific configuration fields:
   - `is_glic_eligible`: `false` → `true`
   - `variations_country`: `<current>` → `"us"`
   - `variations_permanent_consistency_country`: `<current>` → `["us"]`
4. **Validating** the modified JSON to ensure Chrome can read it
5. **Logging** all operations for debugging

### Why This Works

Chrome checks these configuration fields to determine Gemini availability. By setting them to US region values, Gemini features become unlocked even if you're in a different region.

**Important Notes:**
- This only modifies local configuration
- No network requests are made
- No data is sent to external servers
- Changes are reversible via the `--restore` flag

---

## 🏗️ Building from Source

### Prerequisites

- Rust 1.85.0 or later (with Rust 2024 edition support)
- Cargo

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- --help
```

The release binary will be at `target/release/gemini-unlock`.

---

## 🧪 Testing

We maintain 100% test coverage with 22+ tests:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_complete_patch_workflow

# Show test coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

Test suites include:
- **Unit Tests**: 13 tests for core functionality
- **Integration Tests**: 8 tests for end-to-end workflows
- **Documentation Tests**: 1 test for code examples

---

## 📁 File Locations

The tool operates on Chrome's Local State file:

| Platform | Location |
|----------|----------|
| **macOS** | `~/Library/Application Support/Google/Chrome/Local State` |
| **Linux** | `~/.config/google-chrome/Local State` |
| **Windows** | `%LOCALAPPDATA%\Google\Chrome\User Data\Local State` |

A backup is created as `Local State.bak` in the same directory.

---

## 🛡️ Safety

This tool is designed with safety in mind:

✅ **Type-Safe JSON Parsing** - Uses `serde_json`, not regex
✅ **Input Validation** - Verifies JSON before and after modification
✅ **Automatic Backups** - Creates `.bak` file before changes
✅ **Process Detection** - Refuses to run if Chrome is open
✅ **Zero Network** - No external connections
✅ **Open Source** - Fully auditable code
✅ **Reversible** - Easy restore with `--restore` flag

---

## 💡 Inspiration

This project was inspired by and built upon ideas from:
- **[Gemini-in-Chrome](https://github.com/appsail/Gemini-in-Chrome)** by appsail - Original concept and implementation

Special thanks to the original author for the pioneering work on enabling Chrome Gemini features. This Rust implementation aims to provide a safer, cross-platform, and more maintainable solution with comprehensive testing and type-safe JSON handling.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Clone your fork
git clone https://github.com/flyingzl/gemini-unlock.git
cd gemini-unlock

# Install development dependencies
cargo install cargo-tarpaulin  # For coverage reports

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt

# Run tests
cargo test
```

---

## 📄 License

This project is licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)

at your option.

---

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [serde](https://serde.rs/) and [serde_json](https://github.com/serde-rs/json)
- CLI powered by [clap](https://github.com/clap-rs/clap)
- Error handling with [thiserror](https://github.com/dtolnay/thiserror) and [anyhow](https://github.com/dtolnay/anyhow)

---

## 📚 Related Projects

- **[Gemini-in-Chrome](https://github.com/appsail/Gemini-in-Chrome)** by appsail - Original implementation
- [ungoogled-chromium](https://github.com/ungoogled-software/ungoogled-chromium) - Chromium without Google integration
- [chromium](https://github.com/chromium/chromium) - Chromium source code

---

## 📞 Support

- 🐛 [Report Bugs](https://github.com/flyingzl/gemini-unlock/issues/new?template=bug_report.md)
- 💡 [Request Features](https://github.com/flyingzl/gemini-unlock/issues/new?template=feature_request.md)
- ❓ [Ask Questions](https://github.com/flyingzl/gemini-unlock/issues/new?template=question.md)

---

<div align="center">

**Made with ❤️ and Rust by [@flyingzl](https://github.com/flyingzl)**

⭐ Star this repo if it helped you!

</div>
