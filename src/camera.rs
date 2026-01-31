use anyhow::Result;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use std::{sync::Arc, thread, time::Duration};
use tokio::sync::watch;

pub type FrameReceiver = watch::Receiver<Option<Arc<Vec<u8>>>>;

pub fn start_camera_capture(index: u32, width: u32, height: u32, fps: u32, request_format: &str) -> Result<FrameReceiver> {
    let camera = initialize_camera(index, width, height, fps, request_format)?;
    let (tx, rx) = watch::channel(None);

    thread::spawn(move || {
        if let Err(e) = run_capture_loop(tx, camera) {
            eprintln!("Camera capture failed: {:?}", e);
        }
    });

    Ok(rx)
}

fn initialize_camera(
    index: u32,
    width: u32,
    height: u32,
    fps: u32,
    request_format: &str,
) -> Result<Camera> {
    let index = CameraIndex::Index(index);
    
    let frame_format = match request_format {
        "MJPEG" => nokhwa::utils::FrameFormat::MJPEG,
        "YUYV" => nokhwa::utils::FrameFormat::YUYV,
        "GRAY" => nokhwa::utils::FrameFormat::GRAY,
        "RAWRGB" => nokhwa::utils::FrameFormat::RAWRGB,
        "RGB24" => nokhwa::utils::FrameFormat::RAWRGB,
        _ => {
            eprintln!("Unknown format {}, defaulting to MJPEG", request_format);
            nokhwa::utils::FrameFormat::MJPEG
        }
    };

    let target_format = nokhwa::utils::CameraFormat::new(
        nokhwa::utils::Resolution::new(width, height),
        frame_format,
        fps,
    );

    let mut camera = {
        let req = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(target_format));
        Camera::new(index.clone(), req)?
    };

    camera.open_stream()?;
    Ok(camera)
}

fn run_capture_loop(
    tx: watch::Sender<Option<Arc<Vec<u8>>>>,
    mut camera: Camera,
) -> Result<()> {


    println!("Camera started: {:?}", camera.camera_format());
    let use_mjpeg = camera.camera_format().format() == nokhwa::utils::FrameFormat::MJPEG;

    loop {
        // Try to get a frame
        match camera.frame() {
            Ok(buffer) => {
                let jpeg_data = if use_mjpeg {
                    buffer.buffer().to_vec()
                } else {
                    // Buffer is ImageBuffer<Rgb<u8>, Vec<u8>>
                    // We need to encode this to JPEG for the stream
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
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("Failed to encode JPEG: {}", e);
                            continue;
                        }
                    }
                    jpeg_data
                };
                
                let _ = tx.send(Some(Arc::new(jpeg_data)));
            }
            Err(e) => {
                eprintln!("Failed to get frame: {}", e);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
