# Dockerpage

Dockerpage is a Ruby gem that provides a terminal user interface (TUI) for managing Docker containers.

## Features

- List all Docker containers
- Display container details including name, image, IP address, and ports
- Select and interact with containers using keyboard shortcuts
- Copy container IP address to clipboard
- Open container web interface in browser
- Stop selected container

## Installation

Add this line to your application's Gemfile:

```ruby
gem 'dockerpage'
```

And then execute:

```
$ bundle install
```

Or install it yourself as:

```
$ gem install dockerpage
```

## Usage

Run the `dockerpage` command to launch the TUI:

```
$ dockerpage
```

### Keyboard shortcuts

- `j`: Move selection down
- `k`: Move selection up
- `c`: Copy selected container's IP address to clipboard
- `l`: Open selected container's web interface in browser
- `x`: Stop selected container
- `q`: Quit the application

## Development

After checking out the repo, run `bin/setup` to install dependencies. You can also run `bin/console` for an interactive prompt that will allow you to experiment.

To install this gem onto your local machine, run `bundle exec rake install`.

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/[USERNAME]/dockerpage.

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
