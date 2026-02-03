# Pi Webcam Streamer

Raspberry Pi 用に設計された、軽量で高性能なウェブカメラ映像ストリーミング API サーバーです。**Rust** で記述されています。V4L2 と FFmpeg を使用したビデオストリーミングと H.264 録画機能を備えています。

## 概要

USB ウェブカメラからの映像をキャプチャし、HTTP 経由でストリーミングするミニマリストな API サーバーです。また、バックグラウンドでの連続セグメント録画もサポートしています。



## 特徴

- ウェブカメラからのリアルタイム映像キャプチャ
- HTTP 経由の Motion JPEG (MJPEG) ストリーミング
- バックグラウンドでの H.264 セグメント録画 (MP4)
- `config.toml` ファイルによる設定
- **クロスプラットフォーム**: Linux (Raspberry Pi V4L2) および macOS (AVFoundation) で動作し、開発が容易です。

## 技術スタック

- **Rust**: プログラミング言語
- **Axum**: Web フレームワーク
- **Tokio**: 非同期ランタイム
- **Nokhwa**: クロスプラットフォーム対応カメラキャプチャライブラリ
- **FFmpeg**: H.264 エンコードと録画に使用

## API 仕様

### エンドポイント

#### `GET /`

ウェブカメラのストリームを表示するプレーヤーを含む HTML ページを返します。

#### `GET /stream`

ウェブカメラの映像を MJPEG 形式 (`multipart/x-mixed-replace`) でストリーミングします。

## セットアップ

### 前提条件

- **Rust ツールチェーン**: [rustup.rs](https://rustup.rs) からインストールしてください。
- **FFmpeg**: H.264 録画に必要です。
    - macOS: `brew install ffmpeg`
    - Raspberry Pi: `sudo apt install ffmpeg`

### 開発 (macOS)

1. リポジトリをクローンします。
2. `config.toml` ファイルを作成します (設定を参照)。
3. サーバーを実行します:
   ```bash
   cargo run
   ```

### デプロイ (Raspberry Pi)

1. Raspberry Pi (ARM64) 用にクロスコンパイルします:
   ```bash
   # ターゲットの追加
   rustup target add aarch64-unknown-linux-gnu
   
   # ビルド (リンカーの設定が必要な場合があります。最も簡単なのは Pi 上で直接ビルドすることです)
   cargo build --release --target aarch64-unknown-linux-gnu
   ```
   *注意: クロスコンパイル環境の構築が面倒な場合は、Raspberry Pi 4/5 上で直接ビルドすることをお勧めします。*

2. バイナリを実行します:
   ```bash
   ./target/release/pi-webcam-streamer
   ```

## 設定 (config.toml)

プロジェクトディレクトリに `config.toml` ファイルを作成して設定をカスタマイズします。

```toml
# カメラ設定
camera_index = 0          # 例: /dev/video0 または 0
camera_width = 320        # デフォルト: 320
camera_height = 240       # デフォルト: 240
camera_fps = 5            # デフォルト: 5

# サーバー設定
port = 8080               # デフォルト: 8080

# 録画設定 (任意)
# recording_path が設定されている場合、バックグラウンド録画が有効になります。
recording_path = "./recordings"
recording_segment_minutes = 10
```

## サービスとして実行 (systemd)

本番環境では systemd サービスとして実行してください。

1. バイナリを `/opt/pi-webcam-streamer` にインストールします。
2. `config.toml` を `/etc/pi-webcam-streamer/config.toml` または同じディレクトリにコピーします。
   アプリケーションは以下の順序で設定を探します:
   - `/etc/pi-webcam-streamer/config.toml`
   - `./config.toml`
3. サービスファイル `/etc/systemd/system/pi-webcam-streamer.service` を作成します:

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
