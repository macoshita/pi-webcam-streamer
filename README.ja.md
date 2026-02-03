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

1. **クロスコンパイル (ARM64)**:
   ```bash
   rustup target add aarch64-unknown-linux-gnu
   cargo build --release --target aarch64-unknown-linux-gnu
   ```

2. **インストール**:
   バイナリを `/usr/local/bin` (または PATH の通ったディレクトリ) に配置します:
   ```bash
   sudo cp ./target/aarch64-unknown-linux-gnu/release/pi-webcam-streamer /usr/local/bin/
   ```

3. **設定**:
   ```bash
   sudo mkdir -p /etc/pi-webcam-streamer
   sudo cp config.toml /etc/pi-webcam-streamer/
   ```

4. **サービス管理 (systemd)**:
   組み込みコマンドを使用して簡単にサービス登録・管理ができます:
   ```bash
   # サービス登録 (sudo が必要)
   sudo pi-webcam-streamer service install
   
   # 確認
   sudo systemctl status pi-webcam-streamer
   
   # サービス削除 (sudo が必要)
   sudo pi-webcam-streamer service uninstall
   ```
   `service install` コマンドにより、`/etc/systemd/system/pi-webcam-streamer.service` にユニットファイルが自動生成され、サービスが開始されます。

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
