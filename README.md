# <p align="center"> nmrs-tui 🦀 </p>

<p align="center">
  <strong>A fast, aesthetic, and Vim-friendly NetworkManager TUI built in Rust.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-heavy%20development-orange" alt="Status">
  <img src="https://img.shields.io/badge/license-GPL--3-blue" alt="License">
  <img src="https://img.shields.io/badge/built%20with-Ratatui-red" alt="Ratatui">
</p>

---
> [!IMPORTANT]
> `nmrs-tui` is currently in its early stages. Expect breaking changes and frequent updates as we move toward a stable release.

`nmrs-tui` is a terminal user interface for [nmrs](https://github.com/cachebag/nmrs), inspired from [impala](https://github.com/pythops/impala).

## ✨ Features
- **Available Networks:** Real-time scanning and listing of WiFi networks.
- **Known Networks:** Easily manage and connect to your saved profiles (WIP).
- **Device Management:** View and control your network interfaces (WIP).
- **Customizable:** (WIP).
- **Vim-like Keybindings:** Navigate your networks with Vim-like keybinds.

## 🚀 Installation

### Prerequisites
- **NetworkManager** must be installed and running.
- **Rust** if you are building from source.

### Build from Source
```bash
git clone https://github.com/y2w8/nmrs-tui.git
cd nmrs-tui
cargo build --release
```
The binary will be available at `target/release/nmrs-tui`.

## ⌨️ Keybindings
| Key | Action |
| ------------- | -------------- |
| `Shift+Tab`/`Tab`/`h`/`l` | Switch between tabs (Known/Available/Devices) |
| `↓`/`j` | Move selection down |
| `↑`/`k` | Move selection up |
| `Enter`/`Space` | Connect to selected network |
| `Esc` | Cancel / Close popup |
| `q` | Quit application |


## 🛠️ Debugging & Logs
If you encounter issues, `nmrs-tui` provides detailed logging. You can set the log level using the `RUST_LOG` environment variable:
```bash
RUST_LOG=debug nmrs-tui
```
Logs are stored in your OS cache directory (e.g., `~/.cache/nmrs-tui/nmrs-tui.log`).

## 🏗️ Project Structure

- `src/main.rs`: Entry point.
- `src/tui.rs`: Main loop.
- `src/ui/`: UI components including `table`, `popup`, and `input` logic.
- `src/events.rs`: Keyboard event handling and application logic.
- `src/network_manager.rs`: Integration with the `nmrs` backend.
- `src/logger.rs`: Logging implementation.

## 🤝 Contributing

Contributions are welcome! Whether it's a bug report, a new feature idea, or a pull request, feel free to open an issue or submit a PR.
