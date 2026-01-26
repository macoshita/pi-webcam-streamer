# Pi Webcam Streamer

A lightweight, high-performance webcam video streaming API server specifically designed for Raspberry Pi, written in **Rust**. It features video streaming and H.264 recording via V4L2 and FFmpeg.

## Overview

This is a minimalist API server that captures video from a commercial webcam connected to a Raspberry Pi and streams it via HTTP. It also supports continuous segmented background recording.



## Features

- Real-time video capture from webcam
- Motion JPEG (MJPEG) streaming via HTTP
- H.264 Segmented Recording (MP4) in the background
- Configurable settings via `.env` file
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
2. Create a `.env` file (see Configuration).
3. Run the server:
   ```bash
   cargo run
   ```

### Deployment (Raspberry Pi)

1. Cross-compile for Raspberry Pi (ARM64):
   ```bash
   # Add target
   rustup target add aarch64-unknown-linux-gnu
   
   # Build (you may need a linker, or simpler: build on the Pi itself)
   cargo build --release --target aarch64-unknown-linux-gnu
   ```
   *Note: Building directly on the Raspberry Pi 4/5 is often the easiest way if you don't want to set up a cross-compiler toolchain.*

2. Run the binary:
   ```bash
   ./target/release/pi-webcam-streamer
   ```

## Configuration (.env)

Create a `.env` file in the project directory to customize settings.

```bash
# Camera Settings
CAMERA_INDEX=0          # e.g., /dev/video0 or 0
CAMERA_WIDTH=320        # Default: 320
CAMERA_HEIGHT=240       # Default: 240
CAMERA_FPS=5            # Default: 5

# Server Settings
PORT=8080               # Default: 8080

# Recording Settings (Optional)
# If RECORDING_PATH is set, background recording is enabled.
RECORDING_PATH=./recordings
RECORDING_SEGMENT_MINUTES=10
```

## Running as a Service (systemd)

For production use, run as a systemd service.

1. Install binary to `/opt/pi-webcam-streamer`.
2. Copy `.env` to the same directory.
3. Create service file `/etc/systemd/system/pi-webcam-streamer.service`:

```ini
[Unit]
Description=Pi Webcam Streamer Service
After=network.target

[Service]
Type=simple
User=pi
Group=video
WorkingDirectory=/opt/pi-webcam-streamer
ExecStart=/opt/pi-webcam-streamer/pi-webcam-streamer
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```
