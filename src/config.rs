use serde::Deserialize;
use config::{Config as ConfigLoader, File, Environment};


#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub camera_index: u32,
    pub camera_width: u32,
    pub camera_height: u32,
    pub camera_fps: u32,
    pub port: u16,
    pub camera_format: String,
    pub recording_path: Option<String>,
    pub recording_enable_hls: bool,
    pub recording_segment_seconds: u64,
    pub recording_video_codec: Option<String>,
    pub recording_retention_days: u64,
    pub recording_max_size_mb: u64,
}

impl Config {
    pub fn load() -> Self {


        let builder = ConfigLoader::builder()
            // 1. Default values
            .set_default("camera_index", 0).unwrap()
            .set_default("camera_width", 320).unwrap()
            .set_default("camera_height", 240).unwrap()
            .set_default("camera_fps", 5).unwrap()
            .set_default("port", 8080).unwrap()
            .set_default("camera_format", "MJPEG").unwrap()
            .set_default("camera_format", "MJPEG").unwrap()
            // recording_path is Option, so no default means None
            .set_default("recording_enable_hls", true).unwrap()
            .set_default("recording_segment_seconds", 10).unwrap()
            // recording_video_codec is Option, so no default means None
            .set_default("recording_retention_days", 7).unwrap()
            .set_default("recording_max_size_mb", 128).unwrap()
            
            // 2. System-wide config file
            .add_source(File::with_name("/etc/pi-webcam-streamer/config.toml").required(false))
            
            // 3. Local config file
            .add_source(File::with_name("config.toml").required(false))
            
            // 4. Environment variables
            // Maps APP_CAMERA_INDEX to camera_index, etc.
            // Also supports existing env vars if we map them manually or just use raw env lookup in older way,
            // but config::Environment can map prefixes. 
            // Let's try to support the old names directly for backward compatibility if possible,
            // or just rely on the fact that we might not strictly need the old env vars if we switch to config.
            // However, to be safe and compatible with the user's existing .env logic which sets "CAMERA_INDEX" etc (uppercase),
            // we can tell config to look for those.
            .add_source(Environment::default().try_parsing(true).separator("_"));

        // Note: The Environment::default() above works well for things like "PORT".
        // Use `try_parsing(true)` to parse numbers.

        let config = builder.build().unwrap();
        
        // We need to handle the case where some fields might be missing if relying purely on Deserialize
        // but since we set defaults for everything except Options, it should be fine.
        // However, `config` crate might error if type mismatch.
        
        match config.try_deserialize() {
            Ok(c) => c,
            Err(e) => {
                // If deserialization fails, it might be due to missing required fields or type errors.
                // Fallback to manual construction or panic with a helpful message.
                // For now, let's just panic as configuration is critical.
                panic!("Failed to load configuration: {}", e);
            }
        }
    }
}

