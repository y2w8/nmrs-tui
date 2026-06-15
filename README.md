# <p align="center"> nmrs-tui 🦀 </p>

<p align="center">
  <strong>A fast, keyboard-first TUI for NetworkManager — built in Rust.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-BETA-orange?style=for-the-badge" alt="Status">
  <img src="https://img.shields.io/badge/license-GPL--3-blue?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/built%20with-Ratatui-red?style=for-the-badge" alt="Ratatui">
</p>

---

`nmrs-tui` is a terminal user interface for [nmrs](https://github.com/cachebag/nmrs), inspired from [impala](https://github.com/pythops/impala).

## ✨ Features
- **Available Networks:** Real-time scanning and listing of WiFi networks.
- **Known Networks:** Easily manage and connect to your saved profiles.
- **Device Management:** View and control your network interfaces (WIP).
- **Customizable:** (WIP).
- **Vim-like Keybindings:** Navigate your networks with Vim-like keybinds.

## 🚀 Installation

### Prerequisites
- [**NetworkManager**](https://wiki.archlinux.org/title/NetworkManager) must be installed and running.
- **Rust** if you are building from source.

### 📦 Cargo
```sh
cargo install nmrs-tui
```
The binary will be available at `~/.cargo/bin/nmrs-tui`.
Make sure to add path `export PATH="$HOME/.cargo/bin:$PATH"`

### Build from Source
```bash
git clone https://github.com/y2w8/nmrs-tui.git
cd nmrs-tui
cargo build --release
```
The binary will be available at `./target/release/nmrs-tui`.

## 🛠️ Debugging & Logs
If you encounter issues, `nmrs-tui` provides detailed logging. You can set the log level using the `NMRS_LOG` environment variable:
```bash
NMRS_LOG=trace nmrs-tui
```
Logs are stored in your cache directory `~/.cache/nmrs-tui/nmrs-tui.log`, fallback to cwd.

## 🤝 Contributing

Contributions are welcome! Whether it's a bug report, a new feature idea, or a pull request, feel free to open an issue or submit a PR.
