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
        Self {
            config,
            frame_rx,
            fps,
        }
    }

    pub fn start(mut self) {
        tokio::spawn(async move {
            let recording_path = match &self.config.recording_path {
                Some(path) => path,
                None => return, // No recording path configured
            };

            if let Err(e) = tokio::fs::create_dir_all(recording_path).await {
                eprintln!("Failed to create recording directory: {}", e);
                return;
            }

            // Start PlaylistManager in background
            PlaylistManager::start(self.config.clone(), recording_path.clone());

            let mut process: Option<Child> = None;

            loop {
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
        let segment_time = self.config.recording_segment_seconds.to_string();
        let segment_pattern = format!("{}/segment_%09d.mp4", path);
        let hls_list_size = self.config.hls_list_size().to_string();

        let mut cmd = Command::new("ffmpeg");

        // Log level
        cmd.args(&["-loglevel", "error"]);

        // Input
        cmd.args(&[
            "-use_wallclock_as_timestamps",
            "1",
            "-f",
            "mjpeg",
            "-i",
            "pipe:0",
        ]);

        // Output Format & Codec
        cmd.args(&["-vf", "format=yuv420p"]);
        cmd.args(&["-r", &fps]); // Normalize output framerate

        let video_codec =
            self.config
                .recording_video_codec
                .as_deref()
                .unwrap_or(if cfg!(target_os = "linux") {
                    "h264_v4l2m2m"
                } else {
                    "libx264"
                });

        cmd.args(&["-c:v", video_codec]);

        // GOP: keyframe every 2 seconds, aligns well with segmentation
        let gop = (self.config.camera_fps * 2).to_string();
        cmd.args(&["-g", &gop]);

        if video_codec == "h264_v4l2m2m" {
            // Hardware encoding on Pi
            cmd.args(&["-b:v", "5M"]);
        } else if video_codec == "libx264" {
            // Software encoding
            cmd.args(&["-preset", "ultrafast"]);
        }

        // HLS output: FFmpeg manages playlist and deletes old segments natively.
        let playlist_path = format!("{}/playlist.m3u8", path);
        cmd.args(&[
            "-f",
            "hls",
            "-hls_time",
            &segment_time,
            "-hls_segment_type",
            "fmp4",
            "-hls_flags",
            "independent_segments+append_list+delete_segments+program_date_time",
            "-hls_segment_filename",
            &segment_pattern,
            "-hls_list_size",
            &hls_list_size,
            &playlist_path,
        ]);

        cmd.stdin(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        cmd.spawn()
    }
}

struct PlaylistManager;

impl PlaylistManager {
    fn start(config: Config, recording_path: String) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                config.recording_segment_seconds / 2, // run twice per segment at least
            ));

            loop {
                interval.tick().await;

                if let Err(e) = Self::manage_playlist(&config, &recording_path).await {
                    eprintln!("Failed to manage playlist: {}", e);
                }
            }
        });
    }

    async fn manage_playlist(config: &Config, path: &str) -> std::io::Result<()> {
        let retention_secs = config.retention_seconds();
        let list_size = config.hls_list_size() as usize;
        let segment_duration = config.recording_segment_seconds as f64;

        let mut dir = tokio::fs::read_dir(path).await?;
        let mut segments = Vec::new();

        let now_timestamp = chrono::Utc::now().timestamp();

        // 1. Gather all .mp4 files and remove old ones
        while let Some(entry) = dir.next_entry().await? {
            let file_type = entry.file_type().await?;
            if !file_type.is_file() {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy().into_owned();
            if !file_name.ends_with(".mp4") || file_name == "init.mp4" {
                continue;
            }

            // Parse UNIX timestamp from filename (e.g. 1771655874.mp4)
            let stem = file_name.trim_end_matches(".mp4");
            if let Ok(segment_time) = stem.parse::<i64>() {
                let age_secs = now_timestamp - segment_time;

                if age_secs > retention_secs as i64 {
                    // Delete file
                    let file_path = entry.path();
                    if let Err(e) = tokio::fs::remove_file(&file_path).await {
                        eprintln!("Failed to delete old segment {:?}: {}", file_path, e);
                    } else {
                        println!("Deleted old segment: {}", file_name);
                    }
                    continue;
                }

                segments.push(file_name);
            }
        }

        // 2. Map and generate playlist
        // Sort segments chronologically
        segments.sort();

        // Keep at most list_size segments in the playlist
        if segments.len() > list_size {
            let start = segments.len() - list_size;
            segments = segments.split_off(start);
        }

        // Ignore empty lists if no segments are recorded yet
        if segments.is_empty() {
            return Ok(());
        }

        let mut playlist_content = String::new();
        playlist_content.push_str("#EXTM3U\n");
        playlist_content.push_str("#EXT-X-VERSION:7\n");
        playlist_content.push_str(&format!(
            "#EXT-X-TARGETDURATION:{}\n",
            config.recording_segment_seconds
        ));

        let mut sequence = 0;
        if let Some(first_file) = segments.first() {
            // Derive sequence number roughly from timestamp for continuity
            let stem = first_file.trim_end_matches(".mp4");
            if let Ok(segment_time) = stem.parse::<i64>() {
                sequence = segment_time / config.recording_segment_seconds as i64;
            }
        }

        playlist_content.push_str(&format!("#EXT-X-MEDIA-SEQUENCE:{}\n", sequence));
        playlist_content.push_str("#EXT-X-PLAYLIST-TYPE:EVENT\n");

        // We write init.mp4 only if we are creating fmp4
        playlist_content.push_str("#EXT-X-MAP:URI=\"init.mp4\"\n");

        for segment in segments {
            // Currently hardcoding duration based on configuration instead of parsing files
            playlist_content.push_str(&format!("#EXTINF:{:.4},\n", segment_duration));
            playlist_content.push_str(&format!("{}\n", segment));
        }

        // 3. Write to temporary file and rename to avoid partial reads
        let temp_playlist = format!("{}/playlist.m3u8.tmp", path);
        let final_playlist = format!("{}/playlist.m3u8", path);

        tokio::fs::write(&temp_playlist, playlist_content).await?;
        tokio::fs::rename(&temp_playlist, &final_playlist).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manage_playlist_retention() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().to_str().unwrap();

        // Config setup: 5 hours retention
        let mut config = Config::default();
        config.recording_retention = "5h".to_string();
        config.recording_segment_seconds = 10;

        let now_ts = chrono::Utc::now().timestamp();

        // File 1: 10 hours ago (should be deleted)
        let time_10h_ago = now_ts - (10 * 3600);
        let file_10h_ago = format!("{}.mp4", time_10h_ago);
        tokio::fs::write(temp_dir.path().join(&file_10h_ago), b"dummy")
            .await
            .unwrap();

        // File 2: 1 hour ago (should be kept)
        let time_1h_ago = now_ts - (1 * 3600);
        let file_1h_ago = format!("{}.mp4", time_1h_ago);
        tokio::fs::write(temp_dir.path().join(&file_1h_ago), b"dummy")
            .await
            .unwrap();

        // Run manage_playlist
        PlaylistManager::manage_playlist(&config, path)
            .await
            .unwrap();

        // Check which files exist
        let mut dir = tokio::fs::read_dir(path).await.unwrap();
        let mut files = Vec::new();
        while let Some(entry) = dir.next_entry().await.unwrap() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".mp4") {
                files.push(name);
            }
        }

        assert!(
            !files.contains(&file_10h_ago),
            "Old file {} should be deleted",
            file_10h_ago
        );
        assert!(
            files.contains(&file_1h_ago),
            "New file {} should be kept",
            file_1h_ago
        );

        // Verify playlist.m3u8 content
        let playlist_path = temp_dir.path().join("playlist.m3u8");
        let playlist = tokio::fs::read_to_string(playlist_path).await.unwrap();
        
        let expected_sequence = time_1h_ago / config.recording_segment_seconds as i64;
        let expected_playlist = format!(
            "#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-TARGETDURATION:10\n#EXT-X-MEDIA-SEQUENCE:{}\n#EXT-X-PLAYLIST-TYPE:EVENT\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:10.0000,\n{}\n",
            expected_sequence,
            file_1h_ago
        );

        assert_eq!(playlist, expected_playlist, "The playlist content does not match expected output");
    }
}
