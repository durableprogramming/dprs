# DPRS - Docker Process Management TUI

DPRS is a terminal user interface (TUI) application for managing Docker containers. It provides two main tools:

1. **dprs** - Docker container management tool
2. **dplw** - Docker log watching tool

## Features

### DPRS (Docker Process Manager)
- List all running Docker containers with details
- View container information (name, image, status, IP address, ports)
- Copy container IP address to clipboard
- Open container web interface in browser
- Stop containers
- Refresh container list

### DPLW (Docker Log Watcher)
- Watch logs from all running containers in real-time
- Switch between containers easily
- Scroll through container logs
- Refresh container list

## Requirements

- Rust and Cargo
- Docker

## Installation

```bash
git clone https://github.com/yourusername/dprs.git
cd dprs
./scripts/install.sh
```

The install script will:
1. Check if Cargo and Docker are installed
2. Build the application in release mode
3. Copy the binaries to `~/.local/bin/`

## Usage

### DPRS - Docker Process Manager

Run the application:

```bash
dprs
```

Controls:
- `j`/`↓` - Move selection down
- `k`/`↑` - Move selection up
- `c` - Copy container IP address to clipboard
- `l` - Open container web interface in browser
- `x` - Stop the selected container
- `r` - Refresh container list
- `q` - Quit

### DPLW - Docker Log Watcher

Run the application:

```bash
dplw
```

Controls:
- `←`/`→` - Switch between containers
- `↑`/`↓` - Scroll logs
- `Home` - Scroll to top
- `End` - Scroll to bottom
- `r` - Refresh container list
- `q` - Quit

## License

MIT
