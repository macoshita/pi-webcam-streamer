# Pi Webcam Streamer

Raspberry Pi 用に特別に設計された、軽量で高性能なウェブカメラビデオストリーミングサーバーです。ビデオストリーミング、バックグラウンドでの連続録画、ライブストリームと録画を視聴するためのモダンな Web インターフェースを備えています。

## 概要

このプロジェクトは、Raspberry Pi をネットワークカメラにするための完全なソリューションを提供します。USB ウェブカメラからビデオをキャプチャし、HTTP (MJPEG) 経由でストリーミングし、バックグラウンドで連続した H.264 ビデオセグメントを録画します。付属の Web インターフェースを使用すると、ライブストリームを視聴したり、録画された映像をブラウズ/再生したりできます。

## 機能

- **リアルタイムストリーミング**: HTTP 経由の低遅延 Motion JPEG (MJPEG) ストリーミング。
- **バックグラウンド録画**: 連続した H.264 セグメント録画 (HLS 互換)。
- **Web インターフェース**:
    - ライブビュープレーヤー。
    - 日時選択機能付きの録画ブラウザ。
    - レスポンシブデザイン (モバイルフレンドリー)。
- **高効率**: シングルボードコンピュータに適した低リソース使用量。
- **設定可能**: カメラとサーバーの設定を行うシンプルな `config.toml` ファイル。
- **サービス管理**: systemd サービスのインストール/管理を行う組み込みコマンド。

## 使い方

### 1. インストール

[Releases](https://github.com/macoshita/pi-webcam-streamer/releases) ページから、プラットフォーム用（例: Raspberry Pi 4 `aarch64-unknown-linux-gnu`）の最新リリースをダウンロードしてください。

あるいは、ソースからビルドすることもできます（下の[ビルドと開発](#ビルドと開発)を参照）。

アーカイブを展開します:
```bash
tar -xzf pi-webcam-streamer-*.tgz
cd pi-webcam-streamer
```

### 2. 設定

バイナリと同じディレクトリ、または `/etc/pi-webcam-streamer/config.toml` に `config.toml` ファイルを作成します。

```toml
# カメラ設定
camera_index = 0          # 例: /dev/video0 または 0
camera_width = 640        # 解像度 幅
camera_height = 480       # 解像度 高さ
camera_fps = 30           # フレームレート

# サーバー設定
port = 8080               # Webサーバーのポート

# 録画設定 (オプション)
# recording_path が設定されている場合、バックグラウンド録画が有効になります。
recording_path = "./recordings"
recording_segment_minutes = 10
```

### 3. サーバーの実行

**手動実行:**
```bash
./pi-webcam-streamer
```

**サービスとして実行 (systemd):**
アプリケーションには、systemd サービスを管理するための組み込みコマンドが含まれています。

```bash
# サービスのインストール (sudo が必要)
# これにより /etc/systemd/system/pi-webcam-streamer.service が作成されます
sudo ./pi-webcam-streamer service install

# サービスの開始
sudo systemctl start pi-webcam-streamer

# ステータスの確認
sudo systemctl status pi-webcam-streamer

# サービスの停止
sudo systemctl stop pi-webcam-streamer

# サービスのアンインストール
sudo ./pi-webcam-streamer service uninstall
```

### 4. Web インターフェース

ブラウザを開き、以下にアクセスします:
`http://<your-pi-ip>:8080`

- **ライブビュー**: リアルタイムのカメラ映像を視聴します。
- **録画**: 録画されたビデオセグメントをブラウズして再生します。

## API エンドポイント

- `GET /`: Web UI を配信します。
- `GET /api/stream`: MJPEG ストリーム (`multipart/x-mixed-replace`)。
- `GET /api/videos/*`: 録画されたビデオファイルを配信します (録画が有効な場合)。

---

## ビルドと開発

コードを変更したり、ソースからビルドしたりしたい開発者向けです。

### アーキテクチャ
- **バックエンド**: Rust (Axum, Tokio, Nokhwa)
- **フロントエンド**: SvelteKit, TypeScript, Tailwind CSS, DaisyUI
- **メディア**: FFmpeg (エンコーディング), HLS.js (再生)

### 前提条件

- **Rust ツールチェーン**: [rustup.rs](https://rustup.rs) 経由でインストールします。
- **Bun**: フロントエンドのビルドに必要です。
- **FFmpeg**: H.264 録画のために実行時に必要です。
    - macOS: `brew install ffmpeg`
    - Raspberry Pi: `sudo apt install ffmpeg`

### 1. フロントエンドのビルド
バックエンドは `frontend/build` からフロントエンドの静的ファイルを配信します。まずフロントエンドをビルドする必要があります。
```bash
cd frontend
bun install
bun run build
cd ..
```

### 2. バックエンドのビルド

**開発 (macOS/Linux):**
```bash
cargo run
```

**Raspberry Pi (aarch64) 向けクロスコンパイル:**
```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

### プロジェクト構造
- `src/`: Rust バックエンドのソースコード。
- `frontend/`: SvelteKit フロントエンドのソースコード。
- `frontend/build/`: SvelteKit によって生成された静的ファイル (バックエンドによって配信)。
