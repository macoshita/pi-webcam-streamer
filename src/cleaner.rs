use crate::config::Config;
use std::time::{SystemTime, Duration};
use tokio::fs;

pub struct Cleaner {
    config: Config,
}

impl Cleaner {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn cleanup(&self) {
        let recording_path = match &self.config.recording_path {
            Some(path) => path,
            None => return,
        };

        let max_size_bytes = self.config.recording_max_size_mb * 1024 * 1024;
        let retention_duration = Duration::from_secs(self.config.recording_retention_days * 24 * 60 * 60);

        let mut read_dir = match fs::read_dir(recording_path).await {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to read recording directory: {}", e);
                return;
            }
        };

        let mut files = Vec::new();

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(extension) = path.extension() {
                if extension != "mp4" {
                    continue;
                }
            } else {
                continue;
            }

            let metadata = match entry.metadata().await {
                Ok(m) => m,
                Err(_) => continue,
            };

            let modified = match metadata.modified() {
                Ok(t) => t,
                Err(_) => SystemTime::now(),
            };

            let size = metadata.len();
            // total_size += size; // computed later

            files.push((path, modified, size));
        }

        // Sort by modification time (oldest first)
        files.sort_by(|a, b| a.1.cmp(&b.1));

        let now = SystemTime::now();

        // 1. Retention Check
        let mut kept_files = Vec::new();
        for (path, modified, size) in files {
            let mut keep = true;
            if let Ok(age) = now.duration_since(modified) {
                if age > retention_duration {
                    if let Err(e) = fs::remove_file(&path).await {
                        eprintln!("Failed to delete old file {:?}: {}", path, e);
                    } else {
                        println!("Deleted old file: {:?}", path);
                        keep = false;
                    }
                }
            }
            if keep {
                kept_files.push((path, modified, size));
            }
        }
        files = kept_files;
        
        // Re-calculate total size after retention cleanup
        let mut total_size: u64 = files.iter().map(|(_, _, size)| *size).sum();

        // 2. Size Check
        if total_size > max_size_bytes {
            println!("Total size {} exceeds limit {}, cleaning up...", total_size, max_size_bytes);
            for (path, _, size) in files.iter() {
                if total_size <= max_size_bytes {
                    break;
                }

                if let Err(e) = fs::remove_file(path).await {
                    eprintln!("Failed to delete file for space {:?}: {}", path, e);
                } else {
                    println!("Deleted file for space: {:?}", path);
                    total_size -= size;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cleanup() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap().to_string();

        let config = Config {
            camera_index: 0,
            camera_width: 0,
            camera_height: 0,
            camera_fps: 0,
            port: 0,
            camera_format: "MJPEG".to_string(),
            recording_path: Some(dir_path.clone()),
            recording_segment_minutes: 0,
            recording_video_codec: None,
            recording_retention_days: 1, // 1 day
            recording_max_size_mb: 1, // 1 MB
        };

        let cleaner = Cleaner::new(config);

        // Create 3 files
        // File 1: Old, small
        let file1 = dir.path().join("old.mp4");
        {
            let mut f = std::fs::File::create(&file1).unwrap();
            f.write_all(&[0u8; 1000]).unwrap(); // 1KB
        }
        // Modify time to be 2 days ago
        let old_time = SystemTime::now() - Duration::from_secs(2 * 24 * 3600);
        filetime::set_file_mtime(&file1, filetime::FileTime::from_system_time(old_time)).unwrap();

        // File 2: New, large (2MB) - exceeds 1MB limit alone
        let file2 = dir.path().join("new_large.mp4");
        {
            let mut f = std::fs::File::create(&file2).unwrap();
            f.write_all(&[0u8; 2 * 1024 * 1024]).unwrap(); // 2MB
        }

        // File 3: New, small
        let file3 = dir.path().join("new_small.mp4");
        {
            let mut f = std::fs::File::create(&file3).unwrap();
            f.write_all(&[0u8; 1000]).unwrap(); // 1KB
        }

        // Set mtimes explicitly before cleanup to ensure deterministic behavior
        // File 1: 2 days old (already set above)
        
        // File 2: 60 seconds old
        let time2 = SystemTime::now() - Duration::from_secs(60);
        filetime::set_file_mtime(&file2, filetime::FileTime::from_system_time(time2)).unwrap();

        // File 3: Now
        let time3 = SystemTime::now();
        filetime::set_file_mtime(&file3, filetime::FileTime::from_system_time(time3)).unwrap();

        // Run cleanup
        cleaner.cleanup().await;

        // Verify
        // File 1 should be deleted (retention)
        assert!(!file1.exists(), "File 1 should be gone (retention)");

        // File 2 (2MB) and File 3 (1KB) remain after retention check. 
        // Total size > 1MB.
        // File 2 is older than File 3 (60s vs 0s).
        // Size cleanup should delete File 2 first.
        assert!(!file2.exists(), "File 2 should be gone (size limit, older than file3)");
        assert!(file3.exists(), "File 3 should remain");
    }
}
