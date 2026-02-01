use axum::{
    extract::State,
    response::Html,
    routing::get,
    Router,
};
use chrono::{Local, NaiveDateTime};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod camera;
mod config;
mod recorder;

#[derive(Clone)]
struct AppState {
    frame_rx: camera::FrameReceiver,
    recording_path: Option<String>,
}

#[tokio::main]
async fn main() {
    let config = config::Config::from_env();
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
        recording_path: config.recording_path.clone(),
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

async fn index_handler() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Webcam Stream</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; text-align: center; }
        h1 { color: #333; }
        img { max-width: 100%; border: 2px solid #333; border-radius: 8px; }
        .nav { margin-bottom: 20px; }
        .nav a { margin: 0 10px; color: #007bff; text-decoration: none; }
        .nav a:hover { text_decoration: underline; }
    </style>
</head>
<body>
    <div class="nav">
        <a href="/">Stream</a>
        <a href="/recordings">Recordings</a>
    </div>
    <h1>Webcam Stream</h1>
    <img src="/stream" alt="Webcam Stream">
</body>
</html>"#,
    )
}

async fn recordings_handler(State(state): State<AppState>) -> Html<String> {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Recordings</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; text-align: center; }
        h1 { color: #333; }
        .nav { margin-bottom: 20px; }
        .nav a { margin: 0 10px; color: #007bff; text-decoration: none; }
        .nav a:hover { text_decoration: underline; }
        ul { list-style: none; padding: 0; }
        li { margin: 10px 0; }
        a.file { text-decoration: none; color: #333; font-size: 1.1em; }
        a.file:hover { color: #007bff; }
    </style>
</head>
<body>
    <div class="nav">
        <a href="/">Stream</a>
        <a href="/recordings">Recordings</a>
    </div>
    <h1>Recordings</h1>
    <ul>
"#,
    );

    if let Some(path) = state.recording_path {
        let mut entries = Vec::new();
        if let Ok(mut read_dir) = tokio::fs::read_dir(&path).await {
            while let Ok(Some(entry)) = read_dir.next_entry().await {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Some(stem) = file_name.strip_suffix(".mp4") {
                        if let Ok(dt) = NaiveDateTime::parse_from_str(stem, "%Y%m%d_%H%M%S") {
                            entries.push((file_name, dt));
                        }
                    }
                }
            }
        }
        // Sort descending (newest first)
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        if entries.is_empty() {
             html.push_str("<li>No recordings found.</li>");
        } else {
            let now = Local::now().naive_local();
            for (file_name, dt) in entries {
                let diff = now.signed_duration_since(dt);
                let ago = if diff.num_minutes() < 1 {
                    "たった今".to_string()
                } else if diff.num_minutes() < 60 {
                    format!("{}分前", diff.num_minutes())
                } else if diff.num_hours() < 24 {
                    format!("{}時間前", diff.num_hours())
                } else {
                    format!("{}日前", diff.num_days())
                };
                
                let formatted_date = dt.format("%m月%d日 %H時%M分").to_string();

                html.push_str(&format!(
                    r#"<li><a href="/videos/{}" class="file" target="_blank">{} ({})</a></li>"#,
                    file_name, formatted_date, ago
                ));
            }
        }
    } else {
        html.push_str("<p>Recording is disabled.</p>");
    }

    html.push_str(
        r#"
    </ul>
</body>
</html>"#,
    );

    Html(html)
}
