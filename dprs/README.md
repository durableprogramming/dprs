# DPRS

A Rust implementation of Dockerpage to provide a faster, more efficient terminal user interface (TUI) for managing Docker containers.

## Features

- List all Docker containers with improved performance
- Display container details including name, image, IP address, and ports
- Select and interact with containers using keyboard shortcuts
- Copy container IP address to clipboard
- Open container web interface in browser
- Stop selected container

## Installation

```
cargo install dprs
```

Or build from source:

```
git clone https://github.com/yourusername/dprs
cd dprs
cargo build --release
```

## Usage

Run the `dprs` command to launch the TUI:

```
$ dprs
```

### Keyboard shortcuts

- `j`: Move selection down
- `k`: Move selection up
- `c`: Copy selected container's IP address to clipboard
- `l`: Open selected container's web interface in browser
- `x`: Stop selected container
- `q`: Quit the application

## Why Rust?

The original Dockerpage was written in Ruby, but this Rust implementation offers:

- Significantly faster startup times
- Lower memory usage
- Native binary with no dependencies
- Improved performance when handling many containers

## Development

After checking out the repo, make sure you have the Rust toolchain installed and run:

```
cargo build
```

To run the application locally during development:

```
cargo run
```

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/yourusername/dprs.

## License

The application is available as open source under the terms of the MIT License.
