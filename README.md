# DPRS - Docker Process Management TUI

DPRS (Docker Process Manager) is a terminal user interface for managing Docker containers and monitoring their logs. Built with a focus on reliability, usability, and efficient container management.

## Features

- Container Management
  - List running containers with details (name, image, status, IP, ports)
  - Stop containers with a single keystroke
  - Copy container IP addresses to clipboard
  - Open container web interfaces in browser
  - Real-time container list refresh

- Log Monitoring
  - Real-time log streaming from multiple containers
  - Color-coded log levels (Info, Warning, Error, Debug)
  - Easy navigation between container logs
  - Scroll through log history
  - Automatic log rotation to manage memory usage

## Installation

```bash
cargo install dprs
```

## Usage

DPRS provides two main binaries:

### dprs - Container Manager
```bash
dprs
```
Navigate containers with arrow keys or j/k
- `q`: Quit
- `c`: Copy selected container's IP address
- `l`: Open container web interface in browser
- `x`: Stop selected container
- `r`: Refresh container list

### dplw - Log Watcher
```bash
dplw
```
Watch logs from multiple containers:
- Left/Right arrows: Switch between containers
- Up/Down arrows: Scroll through logs
- Home/End: Jump to start/end of logs
- `r`: Refresh container list
- `q`: Quit

## Philosophy

DPRS is built on principles of:

- Reliability: Stable, well-tested code that handles edge cases gracefully
- Usability: Intuitive interface with clear feedback for all actions
- Efficiency: Fast operation with minimal resource usage
- Pragmatism: Focused on solving real container management needs

## Development

### Requirements
- Rust 2024 edition
- Docker daemon running locally

### Building
```bash
cargo build --release
```

### Testing
```bash
cargo test
```

### Project Structure
- `src/app/`: Application state and action handlers
- `src/display/`: UI components and rendering
- `src/docker_log_watcher/`: Container log monitoring
- `src/log_view/`: Log display and navigation

## Contributing

Contributions are welcome! Please read our contributing guidelines and code of conduct before submitting pull requests.

## License

Copyright (c) 2025 Durable Programming, LLC. All rights reserved.

## Support

For bugs, feature requests, or questions, please open an issue on GitHub.

## Acknowledgments

Built with:
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [clipboard](https://github.com/aweinstock314/rust-clipboard) - Clipboard integration

Special thanks to the Docker and Rust communities for their excellent tools and documentation.
