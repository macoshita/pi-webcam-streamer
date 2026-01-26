use axum::{
    response::Html,
    routing::get,
    Router,
};
use tokio::net::TcpListener;

mod camera;
mod config;
mod recorder;

#[tokio::main]
async fn main() {
    let config = config::Config::from_env();
    println!("Loaded config: {:?}", config);

    // Start camera capture
    let frame_rx = match camera::start_camera_capture(
        config.camera_index,
        config.camera_width,
        config.camera_height,
        config.camera_fps,
    ) {
        Ok(rx) => rx,
        Err(e) => {
            eprintln!("Failed to start camera: {}", e);
            return;
        }
    };

    // Start recorder
    if config.recording_path.is_some() {
        let recorder = recorder::Recorder::new(config.clone(), frame_rx.clone());
        recorder.start();
        println!("Recorder started");
    } else {
        println!("Recorder disabled (RECORDING_PATH not set)");
    }

    // Build our application with a single route
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/stream", get(stream_handler))
        .with_state(frame_rx);

    // Run it with hyper
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn stream_handler(
    axum::extract::State(rx): axum::extract::State<camera::FrameReceiver>,
) -> axum::response::Response {
    use axum::body::Body;
    use futures::stream::StreamExt;

    let stream = tokio_stream::wrappers::WatchStream::new(rx)
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
    </style>
</head>
<body>
    <h1>Webcam Stream</h1>
    <img src="/stream" alt="Webcam Stream">
</body>
</html>"#,
    )
}
