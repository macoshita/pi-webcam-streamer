use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use askama::Template;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use clap::{Parser, Subcommand};

mod camera;
mod config;
mod recorder;
mod cleaner;
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
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

impl IntoResponse for IndexTemplate {
    fn into_response(self) -> Response {
        match self.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err),
            )
                .into_response(),
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
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

    println!("Camera initialized. Configured FPS: {}, Actual FPS: {}", config.camera_fps, actual_fps);

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
    };

    // Build our application
    let mut app = Router::new()
        .route("/", get(index_handler))
        .route("/stream", get(stream_handler))
        .route("/recordings", get(recordings_handler));

    if let Some(path) = config.recording_path {
        app = app.nest_service("/videos", ServeDir::new(path));
    }

    let app = app.with_state(state);

    // Run it with hyper
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn stream_handler(
    State(state): State<AppState>,
) -> axum::response::Response {
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

async fn index_handler() -> IndexTemplate {
    IndexTemplate
}

#[derive(Template)]
#[template(path = "recordings.html")]
struct RecordingsTemplate;

impl IntoResponse for RecordingsTemplate {
    fn into_response(self) -> Response {
        match self.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err),
            )
                .into_response(),
        }
    }
}

async fn recordings_handler() -> RecordingsTemplate {
    RecordingsTemplate
}
