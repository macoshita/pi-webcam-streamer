use axum::{Router, extract::State, routing::get};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use clap::{Parser, Subcommand};

mod camera;
mod config;
mod recorder;
mod service;

#[derive(Parser)]
#[command(name = "pi-webcam-streamer")]
#[command(about = "A webcam streamer for Raspberry Pi", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a default config.toml
    Init,
    /// Manage systemd service
    Service {
        #[command(subcommand)]
        command: ServiceCommands,
    },
}

#[derive(Subcommand)]
enum ServiceCommands {
    /// Install the systemd service
    Install,
    /// Uninstall the systemd service
    Uninstall,
}

#[derive(Clone)]
struct AppState {
    frame_rx: camera::FrameReceiver,
    recording_enabled: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            let config_file_path = "config.toml";
            if std::path::Path::new(config_file_path).exists() {
                eprintln!("Error: config.toml already exists.");
                std::process::exit(1);
            }

            let default_config = config::Config::default();
            let toml_string = format!(
                "# Camera Settings\n\
                camera_index = {}          # e.g., /dev/video0 or 0\n\
                camera_width = {}           # Output video width\n\
                camera_height = {}          # Output video height\n\
                camera_fps = {}           # Frame rate\n\
                camera_format = \"{}\"\n\
                \n\
                # Server Settings\n\
                port = {}               # Web server port\n\
                \n\
                # Recording Settings (Optional)\n\
                # If recording_path is set, background recording is enabled.\n\
                # Note: Continuous recording writes a large amount of data. Writing directly to the SD card may cause it to wear out quickly and fail.\n\
                # We strongly recommend using an external USB drive (HDD/SSD) or a RAM disk for the recording path.\n\
                # recording_path = \"/tmp/recordings\"\n\
                # recording_segment_seconds = {} # Duration of each video segment in seconds\n\
                # recording_video_codec = \"libx264\" # Video codec for recording, e.g., libx264, h264_v4l2m2m (for RPi hardware acceleration)\n\
                # recording_retention = \"{}\" # Retention duration for recordings, e.g. \"12h\", \"1day\", \"7days 12h\"\n",
                default_config.camera_index,
                default_config.camera_width,
                default_config.camera_height,
                default_config.camera_fps,
                default_config.camera_format,
                default_config.port,
                default_config.recording_segment_seconds,
                default_config.recording_retention,
            );

            match std::fs::write(config_file_path, toml_string) {
                Ok(_) => println!("Successfully created config.toml at {}", config_file_path),
                Err(e) => {
                    eprintln!("Error writing config.toml: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Service { command }) => {
            let result = match command {
                ServiceCommands::Install => service::install_service(),
                ServiceCommands::Uninstall => service::uninstall_service(),
            };
            if let Err(e) = result {
                eprintln!("Error: {:#}", e);
                std::process::exit(1);
            }
        }
        None => {
            run_server().await;
        }
    }
}

async fn run_server() {
    let config = config::Config::load();
    println!("Loaded config: {:?}", config);

    // Start camera capture
    let (frame_rx, actual_fps) = match camera::start_camera_capture(
        config.camera_index,
        config.camera_width,
        config.camera_height,
        config.camera_fps,
        &config.camera_format,
    ) {
        Ok((rx, fps)) => (rx, fps),
        Err(e) => {
            eprintln!("Failed to start camera: {}", e);
            // Create a dummy channel so the server can still run
            let (_, rx) = tokio::sync::watch::channel(None);
            (rx, 0)
        }
    };

    println!(
        "Camera initialized. Configured FPS: {}, Actual FPS: {}",
        config.camera_fps, actual_fps
    );

    // Start recorder
    if config.recording_path.is_some() {
        let recorder = recorder::Recorder::new(config.clone(), frame_rx.clone(), actual_fps);
        recorder.start();
        println!("Recorder started with FPS: {}", actual_fps);
    } else {
        println!("Recorder disabled (RECORDING_PATH not set)");
    }

    let state = AppState {
        frame_rx,
        recording_enabled: config.recording_path.is_some(),
    };

    // Build our application with API routes
    let mut app = Router::new()
        .route("/api/stream", get(stream_handler))
        .route("/api/status", get(status_handler));

    if let Some(path) = config.recording_path {
        app = app.nest_service("/api/videos", ServeDir::new(path));
    }

    // Serve SPA frontend from embedded assets
    app = app.fallback(static_handler);

    let app = app.with_state(state);

    // Run it with hyper
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

#[derive(rust_embed::RustEmbed)]
#[folder = "frontend/build/"]
struct Assets;

async fn static_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    use axum::http::{StatusCode, header};
    use axum::response::{IntoResponse, Response};

    let path = uri.path().trim_start_matches('/');

    let (file, content_type) = if path.is_empty() {
        (Assets::get("index.html"), "text/html".to_string())
    } else {
        match Assets::get(path) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                (Some(content), mime.to_string())
            }
            None => {
                // SPA fallback: return index.html if file not found
                // But only if looking for a file that doesn't look like an API call (which are handled by other routes)
                // In this case, since this is a fallback handler, API routes are already handled.
                // However, we should be careful not to return index.html for missing assets (like .js, .css)
                // Usually SPA router handles paths without extensions.
                if path.contains('.') {
                    return (StatusCode::NOT_FOUND, "404 Not Found").into_response();
                }
                (Assets::get("index.html"), "text/html".to_string())
            }
        }
    };

    match file {
        Some(content) => {
            let body = axum::body::Body::from(content.data);
            Response::builder()
                .header(header::CONTENT_TYPE, content_type)
                .body(body)
                .unwrap()
        }
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

#[derive(serde::Serialize)]
struct ServerStatus {
    recording_enabled: bool,
}

async fn status_handler(State(state): State<AppState>) -> axum::Json<ServerStatus> {
    axum::Json(ServerStatus {
        recording_enabled: state.recording_enabled,
    })
}

async fn stream_handler(State(state): State<AppState>) -> axum::response::Response {
    use axum::body::Body;
    use futures::stream::StreamExt;

    let stream = tokio_stream::wrappers::WatchStream::new(state.frame_rx)
        .filter_map(|frame_opt| async move { frame_opt })
        .map(|frame: std::sync::Arc<Vec<u8>>| {
            let header = format!(
                "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                frame.len()
            );
            let mut bytes = Vec::with_capacity(header.len() + frame.len() + 2);
            bytes.extend_from_slice(header.as_bytes());
            bytes.extend_from_slice(&frame);
            bytes.extend_from_slice(b"\r\n");
            Ok::<_, std::io::Error>(bytes)
        });

    let body = Body::from_stream(stream);

    axum::response::Response::builder()
        .header("Content-Type", "multipart/x-mixed-replace; boundary=frame")
        .body(body)
        .unwrap()
}
