use anyhow::{Context, Result};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use std::{sync::Arc, thread, time::Duration};
use tokio::sync::watch;

pub type FrameReceiver = watch::Receiver<Option<Arc<Vec<u8>>>>;

pub fn start_camera_capture(index: u32, width: u32, height: u32, fps: u32) -> Result<FrameReceiver> {
    let (tx, rx) = watch::channel(None);

    thread::spawn(move || {
        let result = run_capture_loop(tx.clone(), index, width, height, fps);
        if let Err(e) = result {
            eprintln!("Camera capture failed: {:?}", e);
        }
    });

    Ok(rx)
}

fn run_capture_loop(
    tx: watch::Sender<Option<Arc<Vec<u8>>>>,
    index: u32,
    width: u32,
    height: u32,
    fps: u32,
) -> Result<()> {
    let index = CameraIndex::Index(index);
    // List cameras for debug
    if let Ok(cameras) = nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
        for cam in cameras {
            println!("Found camera: {:?}", cam);
        }
    }

    let mut camera = {
        let req = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
        Camera::new(index.clone(), req)?
    };

    let mut compatible_formats = camera.compatible_camera_formats()?;
    
    // Filter by requested resolution
    let mut candidates: Vec<_> = compatible_formats
        .iter()
        .cloned()
        .filter(|fmt| fmt.width() == width && fmt.height() == height)
        .collect();

    if candidates.is_empty() {
        println!("No exact resolution match for {}x{}. Using all available formats.", width, height);
        candidates = compatible_formats;
    }

    // Sort by FPS (ascending) to prefer lowest frame rate
    // Then by format (prefer MJPEG over others if FPS is same? Or doesn't matter much)
    candidates.sort_by(|a, b| {
        a.frame_rate().cmp(&b.frame_rate())
            .then_with(|| {
                 // Prefer MJPEG if FPS is equal
                 if a.format() == nokhwa::utils::FrameFormat::MJPEG {
                     std::cmp::Ordering::Less
                 } else if b.format() == nokhwa::utils::FrameFormat::MJPEG {
                     std::cmp::Ordering::Greater
                 } else {
                     std::cmp::Ordering::Equal
                 }
            })
    });

    println!("Found {} candidate formats. Top 3:", candidates.len());
    for (i, fmt) in candidates.iter().take(3).enumerate() {
        println!("{}: {:?}", i + 1, fmt);
    }

    let selected_format = candidates.first().ok_or_else(|| anyhow::anyhow!("No compatible camera formats found"))?;
    
    println!("Selected format: {:?}", selected_format);
    
    camera.set_camera_format(*selected_format)?;
    camera.open_stream()?;


    println!("Camera started: {:?}", camera.camera_format());

    loop {
        // Try to get a frame
        match camera.frame() {
            Ok(buffer) => {
                // Buffer is ImageBuffer<Rgb<u8>, Vec<u8>>
                // We need to encode this to JPEG for the stream
                // TODO: Optimize to get raw MJPEG if possible to avoid re-encoding
                let mut jpeg_data = Vec::new();
                // Use image crate to encode
                // Convert nokhwa Buffer to DynamicImage
                let img = match buffer.decode_image::<RgbFormat>() {
                    Ok(img) => image::DynamicImage::ImageRgb8(img),
                    Err(e) => {
                        eprintln!("Failed to convert buffer: {}", e);
                        continue;
                    }
                };
                let mut cursor = std::io::Cursor::new(&mut jpeg_data);
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 80);
                match encoder.encode_image(&img) {
                    Ok(_) => {
                        let _ = tx.send(Some(Arc::new(jpeg_data)));
                    }
                    Err(e) => eprintln!("Failed to encode JPEG: {}", e),
                }
            }
            Err(e) => {
                eprintln!("Failed to get frame: {}", e);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
