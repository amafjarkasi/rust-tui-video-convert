# 🎬 Rust TUI Video Converter

A powerful terminal-based video converter application built with Rust and Ratatui. Convert your videos between popular formats with a beautiful, intuitive terminal user interface.

![Version](https://img.shields.io/badge/version-1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## ✨ Features

- 📂 **File Browser**: Navigate your filesystem directly in the terminal
- 🔄 **Multiple Format Support**: Convert videos between popular formats (MP4, MKV, AVI, MOV, WEBM)
- 🚀 **Adaptive Conversion**: Automatically uses the best available conversion method:
  - 🧩 Native Rust FFmpeg implementation (when available)
  - 🎮 External FFmpeg (when installed on your system)
  - 🔮 Simulation mode (for demonstration when no converters are available)
- 📊 **Real-time Progress**: Watch your conversion progress with a detailed status display
- ⌨️ **Keyboard Navigation**: Simple and intuitive controls
- 🎨 **Beautiful UI**: Clean, responsive terminal interface with color-coded status indicators
- ⚙️ **Settings Management**: Configure conversion quality and output options

## 🛠️ Requirements

- Rust (stable) 1.65 or newer
- FFmpeg (optional, for hardware-accelerated conversion)

## 📥 Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/rust-tui-video-convert.git
   cd rust-tui-video-convert
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   cargo run --release
   ```

## 🎮 Usage

### Navigation
- `↑`/`↓` or `Tab`: Navigate through files, formats, and tabs
- `Enter`: Select a file or format, or start conversion
- `Tab`: Switch between tabs (File Browser, Format Selection, Settings, Help)

### File Operations
- Navigate to a video file in the File Browser
- Press `Enter` to select it
- Choose your desired output format
- Press `Enter` again to start conversion

### Conversion Controls
- `p`: Toggle popup information
- `n`: Start a new conversion after completion
- `q` or `Esc`: Quit the application or close popups

## 📋 Supported Formats

| Format | Description |
|--------|-------------|
| MP4    | MPEG-4 Part 14 - Widely supported format with good compression |
| MKV    | Matroska Video - Container format that can hold many codecs |
| AVI    | Audio Video Interleave - Microsoft's container format |
| MOV    | QuickTime File Format - Apple's container format |
| WEBM   | WebM - Open, royalty-free format designed for the web |

## 🧩 Architecture

The application is built with a modular architecture:

- **UI Layer**: Built with Ratatui for terminal rendering
- **Application Logic**: Handles state management and user input
- **Conversion Layer**: Supports multiple conversion backends:
  - Native Rust implementation
  - External FFmpeg integration
  - Simulation mode for demonstration

## 🔧 Dependencies

- [ratatui](https://ratatui.rs) - Terminal UI library for building rich interfaces
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [thiserror](https://github.com/dtolnay/thiserror) - Error handling

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📜 License

Distributed under the MIT License. See `LICENSE` for more information.

## 📞 Contact

Your Name - [@your_twitter](https://twitter.com/your_twitter) - email@example.com

Project Link: [https://github.com/yourusername/rust-tui-video-convert](https://github.com/yourusername/rust-tui-video-convert)

---

<p align="center">Made with ❤️ in Rust</p>