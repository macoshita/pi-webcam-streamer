use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub camera_index: u32,
    pub camera_width: u32,
    pub camera_height: u32,
    pub camera_fps: u32,
    pub port: u16,
    pub camera_format: String,
    pub recording_path: Option<String>,
    pub recording_segment_minutes: u32,
}

impl Config {
    pub fn from_env() -> Self {
        // Load .env file if it exists, ignore error if not found
        let _ = dotenvy::dotenv();

        let camera_index = env::var("CAMERA_INDEX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let camera_width = env::var("CAMERA_WIDTH")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(320);

        let camera_height = env::var("CAMERA_HEIGHT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(240);

        let camera_fps = env::var("CAMERA_FPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);

        let camera_format = env::var("CAMERA_FORMAT").unwrap_or_else(|_| "MJPEG".to_string());

        let recording_path = env::var("RECORDING_PATH").ok();

        let recording_segment_minutes = env::var("RECORDING_SEGMENT_MINUTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        Config {
            camera_index,
            camera_width,
            camera_height,
            camera_fps,
            port,
            camera_format,
            recording_path,
            recording_segment_minutes,
        }
    }
}
