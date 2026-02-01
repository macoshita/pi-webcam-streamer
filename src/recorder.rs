use crate::config::Config;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tokio::sync::watch;

pub struct Recorder {
    config: Config,
    frame_rx: watch::Receiver<Option<Arc<Vec<u8>>>>,
    fps: u32,
}

impl Recorder {
    pub fn new(config: Config, frame_rx: watch::Receiver<Option<Arc<Vec<u8>>>>, fps: u32) -> Self {
        Self { config, frame_rx, fps }
    }

    pub fn start(mut self) {
        tokio::spawn(async move {
            let recording_path = match &self.config.recording_path {
                Some(path) => path,
                None => return, // No recording path configured
            };

            // Ensure directory exists
            if let Err(e) = tokio::fs::create_dir_all(recording_path).await {
                eprintln!("Failed to create recording directory: {}", e);
                return;
            }

            let mut process: Option<Child> = None;

            loop {
                // Check signal to change file or just loop forever
                // Actually the receiver loop drives the logic
                
                // If process is not running, start it
                if process.is_none() {
                    match self.spawn_ffmpeg(recording_path) {
                        Ok(child) => {
                            println!("FFmpeg started for recording");
                            process = Some(child);
                        }
                        Err(e) => {
                            eprintln!("Failed to start ffmpeg: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            continue;
                        }
                    }
                }

                // Wait for new frame
                if self.frame_rx.changed().await.is_err() {
                    println!("Frame channel closed, stopping recorder");
                    break;
                }
                
                let frame = self.frame_rx.borrow_and_update().clone();
                if let Some(frame_data) = frame {
                    if let Some(child) = process.as_mut() {
                        if let Some(stdin) = child.stdin.as_mut() {
                            if let Err(e) = stdin.write_all(&frame_data).await {
                                eprintln!("Failed to write to ffmpeg stdin: {}", e);
                                // Kill process and restart
                                let _ = child.kill().await;
                                process = None;
                            }
                        }
                    }
                }
            }
            
            // Cleanup
            if let Some(mut child) = process {
                let _ = child.kill().await;
            }
        });
    }

    fn spawn_ffmpeg(&self, path: &str) -> std::io::Result<Child> {
        let fps = self.fps.to_string();
        let segment_time = (self.config.recording_segment_minutes * 60).to_string();
        let output_pattern = format!("{}/%Y%m%d_%H%M%S.mp4", path);

        let mut cmd = Command::new("ffmpeg");
        
        // Log level
        cmd.args(&["-loglevel", "error"]);

        // Input
        cmd.args(&[
            "-use_wallclock_as_timestamps", "1",
            "-f", "mjpeg",
            "-i", "pipe:0",
        ]);

        // Output Format & Codec
        cmd.args(&["-vf", "format=yuv420p"]);
        cmd.args(&["-r", &fps]); // Normalize output framerate
        
        let video_codec = self.config.recording_video_codec.as_deref().unwrap_or(
            if cfg!(target_os = "linux") { "h264_v4l2m2m" } else { "libx264" }
        );

        cmd.args(&["-c:v", video_codec]);
        
        // Force keyframe every 2 seconds for better streaming/segmentat
        let gop = (self.config.camera_fps * 2).to_string();
        cmd.args(&["-g", &gop]);

        if video_codec == "h264_v4l2m2m" {
            // Hardware encoding on Pi
            cmd.args(&["-b:v", "5M"]);
        } else if video_codec == "libx264" {
            // Software encoding
            cmd.args(&["-preset", "ultrafast"]);
        }

        // Segmentation
        cmd.args(&[
            "-f", "segment",
            "-segment_time", &segment_time,
            "-strftime", "1",
            "-segment_format_options", "movflags=frag_keyframe+empty_moov+default_base_moof",
            &output_pattern,
        ]);

        cmd.stdin(Stdio::piped());
        // cmd.stdout(Stdio::inherit()); // Useful for debug
        cmd.stderr(Stdio::inherit()); 

        cmd.spawn()
    }
}
