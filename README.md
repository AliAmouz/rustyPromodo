# 🍅 Rusty Pomodoro

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://github.com/yourusername/rusty_pomodoro/actions/workflows/rust.yml/badge.svg)](https://github.com/yourusername/rusty_pomodoro/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/rusty_pomodoro.svg)](https://crates.io/crates/rusty_pomodoro)

A powerful, customizable Pomodoro timer with productivity analytics built in Rust.

![Rusty Pomodoro Demo](screenshots/demo.gif)

## ✨ Features

- **Interactive Terminal UI**: Beautiful, responsive TUI using Ratatui
- **Flexible Timer Settings**: Customize work/break intervals to match your workflow
- **Productivity Analytics**: Track and visualize your focus sessions over time
- **Notification System**: Desktop notifications when intervals change
- **Session Management**: Save and resume work sessions
- **Data Export**: Export your productivity data in JSON format
- **Distraction Blocking**: Optional website/app blocking during work sessions

## 🚀 Installation

### From Crates.io

```bash
cargo install rusty_pomodoro
```

### From Source

```bash
git clone https://github.com/yourusername/rusty_pomodoro.git
cd rusty_pomodoro
cargo install --path .
```

## 🔧 Usage

### Quick Start

```bash
# Start a default Pomodoro session (25 min work, 5 min break)
rusty_pomodoro start

# Use custom intervals
rusty_pomodoro start --work 50 --break 10

# Show productivity statistics
rusty_pomodoro stats
```

### Available Commands

```
USAGE:
    rusty_pomodoro [SUBCOMMAND]

SUBCOMMANDS:
    start       Start a new Pomodoro session
    pause       Pause the current session
    resume      Resume a paused session
    reset       Reset the current timer
    stats       Display productivity statistics
    export      Export session data to JSON
    config      Configure timer settings
    help        Print help information
```

## 🧠 Design Philosophy

Rusty Pomodoro was built with the following principles in mind:

1. **Simplicity**: Clean, intuitive interface that doesn't get in your way
2. **Performance**: Low resource usage, even during long work sessions
3. **Extensibility**: Well-structured codebase that's easy to modify and extend
4. **Privacy**: All data stays local - no telemetry or data collection

## 🏗️ Architecture

The project follows a modular architecture:

- **Core**: Timer logic and state management
- **Database**: SQLite storage for session data
- **UI**: Terminal user interface built with Ratatui
- **Analytics**: Session analysis and productivity metrics
- **Notifications**: System notification integration

## 📊 Technical Details

- **Error Handling**: Uses Rust's Result type for robust error management
- **Concurrency**: Leverages Rust's ownership model and threading
- **Database**: Uses SQLite with migrations for data persistence
- **Testing**: Comprehensive unit and integration tests

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Check out the [contributing guidelines](CONTRIBUTING.md) for more details.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📚 Acknowledgements

- The Pomodoro Technique® is a registered trademark of Francesco Cirillo
- Built with [Ratatui](https://github.com/tui-rs-revival/ratatui)
- Inspired by other great productivity tools like [porsmo](https://github.com/ColorCookie-dev/porsmo)
