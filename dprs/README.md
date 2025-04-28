# Docker Process Rust Suite (dprs)

A collection of TUI (Terminal User Interface) tools for managing and working with Docker containers, built with Rust.

## Features

- **Container Management (dprs)**: List, view details, and manage running Docker containers
- **Docker Log Watcher (dplw)**: View and monitor logs from multiple Docker containers in real-time

## Installation

```bash
./scripts/install.sh
```

## Tools

### Docker Process Viewer (dprs)

A terminal-based Docker container manager that provides:

- List of all running containers with details (name, image, status, IP, ports)
- Copy container IP address to clipboard with a single keystroke
- Open web browser to container IP address
- Stop containers
- Refresh container list

#### Usage

```bash
dprs
```

#### Keyboard Shortcuts

- `j` / `↓`: Move selection down
- `k` / `↑`: Move selection up
- `c`: Copy selected container's IP address to clipboard
- `l`: Open selected container's IP address in default web browser
- `x`: Stop selected container
- `r`: Reload container list
- `q`: Quit application

### Docker Process Log Watcher (dplw)

A terminal-based log viewer for Docker containers that provides:

- View logs from all running containers
- Switch between containers with tabs
- Scroll through logs
- Auto-refresh when new log entries are added

#### Usage

```bash
dplw
```

#### Keyboard Shortcuts

- `←` / `→`: Switch between container tabs
- `↑` / `↓`: Scroll through logs
- `Home`: Scroll to top of logs
- `End`: Scroll to bottom of logs
- `r`: Refresh container list
- `q`: Quit application

## Requirements

- Docker installed and running on your system
- For clipboard functionality: xclip/xsel (Linux), pbcopy/pbpaste (macOS)

## License

MIT
