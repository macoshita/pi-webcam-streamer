use anyhow::Result;
use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};
use std::{sync::Arc, thread, time::Duration};
use tokio::sync::watch;

pub type FrameReceiver = watch::Receiver<Option<Arc<Vec<u8>>>>;

pub fn start_camera_capture(
    index: u32,
    width: u32,
    height: u32,
    fps: u32,
    request_format: &str,
) -> Result<(FrameReceiver, u32)> {
    let camera = initialize_camera(index, width, height, fps, request_format)?;
    let actual_fps = camera.camera_format().frame_rate();
    let (tx, rx) = watch::channel(None);

    thread::spawn(move || {
        if let Err(e) = run_capture_loop(tx, camera) {
            eprintln!("Camera capture failed: {:?}", e);
        }
    });

    Ok((rx, actual_fps))
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

fn run_capture_loop(tx: watch::Sender<Option<Arc<Vec<u8>>>>, mut camera: Camera) -> Result<()> {
    println!("Camera started: {:?}", camera.camera_format());

    loop {
        // Try to get a frame
        match camera.frame() {
            Ok(buffer) => {
                // Buffer is ImageBuffer<Rgb<u8>, Vec<u8>>
                // We always encode this to JPEG for the stream to ensure
                // compatibility with iOS Safari, which can be picky about raw MJPEG buffers.
                let mut jpeg_data = Vec::new();

                let img = match buffer.decode_image::<RgbFormat>() {
                    Ok(img) => image::DynamicImage::ImageRgb8(img),
                    Err(e) => {
                        eprintln!("Failed to convert buffer: {}", e);
                        continue;
                    }
                };

                let mut cursor = std::io::Cursor::new(&mut jpeg_data);
                // Lower quality slightly for streaming performance
                let mut encoder =
                    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 80);

                match encoder.encode_image(&img) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Failed to encode JPEG: {}", e);
                        continue;
                    }
                }

                let _ = tx.send(Some(Arc::new(jpeg_data)));
            }
            Err(e) => {
                eprintln!("Failed to get frame: {}", e);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
