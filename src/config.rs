use config::{Config as ConfigLoader, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub camera_index: u32,
    pub camera_width: u32,
    pub camera_height: u32,
    pub camera_fps: u32,
    pub port: u16,
    pub camera_format: String,
    pub recording_path: Option<String>,
    pub recording_segment_seconds: u64,
    pub recording_video_codec: Option<String>,
    /// Retention duration for recordings, e.g. "12h", "1day", "7days 12h"
    pub recording_retention: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            camera_index: 0,
            camera_width: 320,
            camera_height: 240,
            camera_fps: 5,
            port: 8080,
            camera_format: "MJPEG".to_string(),
            recording_path: None,
            recording_segment_seconds: 10,
            recording_video_codec: None,
            recording_retention: "12h".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let default_config = Config::default();
        let builder = ConfigLoader::builder()
            // 1. Default values from Default trait
            .add_source(config::Config::try_from(&default_config).unwrap())
            // 2. System-wide config file
            .add_source(File::with_name("/etc/pi-webcam-streamer/config.toml").required(false))
            // 3. Local config file
            .add_source(File::with_name("config.toml").required(false))
            // 4. Environment variables
            .add_source(Environment::default().try_parsing(true).separator("_"));

        let config = builder.build().unwrap();

        match config.try_deserialize() {
            Ok(c) => c,
            Err(e) => {
                panic!("Failed to load configuration: {}", e);
            }
        }
    }

    /// Parse recording_retention string into seconds using humantime.
    pub fn retention_seconds(&self) -> u64 {
        let duration = humantime::parse_duration(&self.recording_retention).unwrap_or_else(|e| {
            panic!(
                "Invalid recording_retention '{}': {}",
                self.recording_retention, e
            )
        });
        duration.as_secs()
    }

    /// Compute hls_list_size from retention duration and segment time.
    pub fn hls_list_size(&self) -> u64 {
        self.retention_seconds() / self.recording_segment_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hls_list_size() {
        let config = Config {
            camera_index: 0,
            camera_width: 320,
            camera_height: 240,
            camera_fps: 5,
            port: 8080,
            camera_format: "MJPEG".to_string(),
            recording_path: None,
            recording_segment_seconds: 10,
            recording_video_codec: None,
            recording_retention: "1day".to_string(),
        };
        // 86400 / 10 = 8640
        assert_eq!(config.hls_list_size(), 8640);
    }
}
