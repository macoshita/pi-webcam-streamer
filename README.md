# Pi Webcam Streamer

A lightweight, high-performance webcam video streaming API server specifically designed for Raspberry Pi, written in **Rust**. It features video streaming and H.264 recording via V4L2 and FFmpeg.

## Overview

This is a minimalist API server that captures video from a commercial webcam connected to a Raspberry Pi and streams it via HTTP. It also supports continuous segmented background recording.



## Features

- Real-time video capture from webcam
- Motion JPEG (MJPEG) streaming via HTTP
- H.264 Segmented Recording (MP4) in the background
- Configurable settings via `config.toml` file
- **Cross-Platform**: Runs on Linux (Raspberry Pi V4L2) and macOS (AVFoundation) for development.

## Tech Stack

- **Rust**: Programming Language
- **Axum**: Web framework
- **Tokio**: Async runtime
- **Nokhwa**: Cross-platform camera capture library
- **FFmpeg**: Used for efficient H.264 encoding and recording

## API Specification

### Endpoints

#### `GET /`

Returns an HTML page with a player to view the webcam stream.

#### `GET /stream`

Streams the webcam video in MJPEG format (`multipart/x-mixed-replace`).

## Setup

### Prerequisites

- **Rust Toolchain**: Install via [rustup.rs](https://rustup.rs).
- **FFmpeg**: Required for H.264 recording.
    - macOS: `brew install ffmpeg`
    - Raspberry Pi: `sudo apt install ffmpeg`

### Development (macOS)

1. Clone the repository.
2. Create a `config.toml` file (see Configuration).
3. Run the server:
   ```bash
   cargo run
   ```

### Deployment (Raspberry Pi)

1. **Cross-compile for Raspberry Pi (ARM64)**:
   ```bash
   rustup target add aarch64-unknown-linux-gnu
   cargo build --release --target aarch64-unknown-linux-gnu
   ```

2. **Install binary**:
   Move the binary to `/usr/local/bin` (or any directory in your PATH):
   ```bash
   sudo cp ./target/aarch64-unknown-linux-gnu/release/pi-webcam-streamer /usr/local/bin/
   ```

3. **Configure**:
   ```bash
   sudo mkdir -p /etc/pi-webcam-streamer
   sudo cp config.toml /etc/pi-webcam-streamer/
   ```

4. **Service Management (systemd)**:
   Register and manage the service easily using the built-in commands:
   ```bash
   # Install service (requires sudo)
   sudo pi-webcam-streamer service install
   
   # Start/Status
   sudo systemctl status pi-webcam-streamer
   
   # Uninstall service (requires sudo)
   sudo pi-webcam-streamer service uninstall
   ```
   The `service install` command automatically creates a systemd unit file at `/etc/systemd/system/pi-webcam-streamer.service` and enables the service.

## Configuration (config.toml)

Create a `config.toml` file in the project directory to customize settings.

```toml
# Camera Settings
camera_index = 0          # e.g., /dev/video0 or 0
camera_width = 320        # Default: 320
camera_height = 240       # Default: 240
camera_fps = 5            # Default: 5

# Server Settings
port = 8080               # Default: 8080

# Recording Settings (Optional)
# If recording_path is set, background recording is enabled.
recording_path = "./recordings"
recording_segment_minutes = 10
```
