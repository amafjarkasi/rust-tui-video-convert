# 🎬 Rust TUI Video Converter

A powerful terminal-based video converter application built with Rust and Ratatui. This elegant TUI (Terminal User Interface) tool transforms the typically complex process of video conversion into a streamlined, visually appealing experience right in your terminal.

**Rust TUI Video Converter** combines the performance benefits of Rust with the flexibility of multiple conversion backends. Whether you're a content creator needing to convert videos for different platforms, a developer working with multimedia files, or just someone looking to change video formats without the bloat of GUI applications, this tool provides a lightweight yet powerful solution.

The application intelligently adapts to your system's capabilities - using native Rust implementations when available, falling back to external FFmpeg when installed, or running in simulation mode for demonstration purposes. All of this is presented through a beautiful, responsive terminal interface with real-time progress tracking and intuitive keyboard navigation.

![Version](https://img.shields.io/badge/version-1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Rust](https://img.shields.io/badge/rust-1.65%2B-orange)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)

## ✨ Features

- **Intuitive File Browser**: Navigate your filesystem directly in the terminal
- **Multiple Format Support**: Convert to MP4, MKV, AVI, MOV, and WEBM formats
- **Advanced Video Settings**: Control resolution, bitrate, and frame rate for your conversions
- **Adaptive Conversion**: Uses the best available conversion method on your system
- **Real-time Progress**: Track conversion progress with a visual indicator
- **Terminal UI**: Beautiful and responsive interface built with Ratatui

- 🔄 **Comprehensive Format Support**
  - Convert videos between all popular formats:
    - **MP4**: Industry standard with excellent compatibility and compression
    - **MKV**: Versatile container format supporting multiple audio/video tracks
    - **AVI**: Classic format with wide compatibility for older systems
    - **MOV**: Apple's QuickTime format for high-quality video
    - **WEBM**: Open web-friendly format optimized for online streaming
  - Preserves video quality during conversion with configurable settings
  - Maintains metadata where supported by target format

- 🚀 **Smart Adaptive Conversion Engine**
  - Automatically detects and uses the optimal conversion method available on your system:
    - 🧩 **Native Rust Implementation**: Leverages Rust's performance for maximum speed and efficiency
    - 🎮 **External FFmpeg Integration**: Utilizes the power of FFmpeg when installed for hardware acceleration
    - 🔮 **Simulation Mode**: Provides a full demonstration experience when no converters are available
  - Transparent status indicators showing which method is being used
  - Graceful fallback system ensures conversion always works regardless of system configuration

- 📊 **Detailed Real-time Progress Tracking**
  - Live conversion progress bar with percentage completion
  - Current conversion step indicators showing exactly what's happening
  - Estimated time remaining calculations
  - Detailed logging of conversion stages (analyzing, extracting audio, processing video, muxing)
  - Error handling with clear explanations if issues occur

- ⌨️ **Efficient Keyboard-Centric Controls**
  - Fully navigable without a mouse for maximum efficiency
  - Consistent keyboard shortcuts across all screens
  - Tab-based navigation between major application sections
  - Context-sensitive help always available
  - Vim-inspired navigation options for power users

- 🎨 **Polished Terminal User Interface**
  - Clean, responsive design that adapts to terminal size
  - Color-coded status indicators for instant visual feedback
  - Thoughtfully designed layouts for each application screen
  - Consistent visual language throughout the application
  - Attractive borders and styling that works on any terminal
  - High-contrast mode for accessibility

- ⚙️ **Advanced Video Settings**
  - **Resolution Control**: Choose from Original, 720p, 1080p, or 4K output
  - **Bitrate Management**: Select Auto, Low, Medium, or High quality settings
  - **Frame Rate Options**: Maintain original FPS or convert to 24, 30, or 60 FPS
  - **Interactive UI**: Easily adjust settings with keyboard navigation
  - **Visual Feedback**: Highlighted current selection for better usability
  - **Settings Preview**: View your configuration in the conversion dialog

## 🛠️ Requirements

- Rust (stable) 1.65 or newer
- FFmpeg (optional, for hardware-accelerated conversion)

## 📥 Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/amafjarkasi/rust-tui-video-convert.git
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

- `↑`/`↓`: Navigate through files, formats, and settings
- `Enter`: Select a file or format, or start conversion
- `Tab`: Switch between tabs (File Browser, Format Selection, Settings, Help)
- `←`/`→`: Change values in Settings or navigate between tabs

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
  - Responsive layout that adapts to terminal size
  - Tab-based navigation between application sections
  - Interactive widgets for user input and feedback

- **Application Logic**: Handles state management and user input
  - Event-driven design for keyboard interaction
  - State machine for managing application flow
  - Efficient data structures for file and format management

- **Conversion Layer**: Supports multiple conversion backends:
  - Native Rust implementation for maximum performance
  - External FFmpeg integration for hardware acceleration
  - Simulation mode for demonstration purposes

- **Advanced Video Settings**: Configurable options for video conversion:
  - Resolution: Original, 720p, 1080p, 4K
  - Bitrate: Auto, Low, Medium, High
  - Frame Rate: Original, 24fps, 30fps, 60fps

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

Distributed under the MIT License. See LICENSE for more information.

---

Made with ❤️ in Rust