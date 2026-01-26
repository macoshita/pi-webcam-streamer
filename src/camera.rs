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

    let formats_to_try = [
        (640, 480, 30, nokhwa::utils::FrameFormat::MJPEG),
        (640, 480, 30, nokhwa::utils::FrameFormat::YUYV),
        (1280, 720, 30, nokhwa::utils::FrameFormat::MJPEG),
        (1280, 720, 30, nokhwa::utils::FrameFormat::YUYV),
        (320, 240, 30, nokhwa::utils::FrameFormat::MJPEG),
        (320, 240, 30, nokhwa::utils::FrameFormat::YUYV),
    ];

    let mut camera = None;

    for (w, h, f, fmt) in formats_to_try {
        println!("Trying format: {}x{} @ {}fps {:?}", w, h, f, fmt);
        let req = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(
            nokhwa::utils::CameraFormat::new_from(w, h, fmt, f),
        ));
        match Camera::new(index.clone(), req) {
            Ok(mut cam) => {
                if let Ok(_) = cam.open_stream() {
                    println!("Successfully opened camera with {}x{} @ {}fps {:?}", w, h, f, fmt);
                    camera = Some(cam);
                    break;
                }
            }
            Err(e) => {
                println!("Failed to create camera with {:?}: {}", fmt, e);
            }
        }
    }

    let mut camera = match camera {
        Some(c) => c,
        None => {
             println!("All requested formats failed. Trying defaults as last resort...");
             let requested_auto = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
             let mut cam = Camera::new(index, requested_auto).context("Failed to create camera with default settings")?;
             cam.open_stream().context("Failed to open camera stream with default settings")?;
             cam
        }
    };


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
