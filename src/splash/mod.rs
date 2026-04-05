#[cfg(feature = "splash")]
mod window;

#[cfg(feature = "splash")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "splash")]
use std::sync::Arc;
#[cfg(feature = "splash")]
use std::thread;

/// Run a closure while displaying the splash screen.
/// The splash runs on the main thread (required by macOS),
/// and the work closure runs on a background thread.
/// Returns the result of the work closure after the splash closes.
#[cfg(feature = "splash")]
pub fn run_with_splash<F, R>(work: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let image_bytes: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/splash/embedded_logo.bin"
    ));

    if image_bytes.is_empty() {
        return work();
    }

    // Decode image
    let img = match image::load_from_memory(image_bytes) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Splash: failed to decode image: {}", e);
            return work();
        }
    };

    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();

    // Convert RGBA to packed u32 (0x00RRGGBB) for softbuffer
    let pixels: Vec<u32> = rgba
        .chunks_exact(4)
        .map(|px| ((px[0] as u32) << 16) | ((px[1] as u32) << 8) | (px[2] as u32))
        .collect();

    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();

    // Channel to get the work result back from the background thread
    let (result_tx, result_rx) = std::sync::mpsc::channel();

    // Spawn the bootstrap work on a background thread
    thread::spawn(move || {
        let result = work();
        done_clone.store(true, Ordering::Relaxed);
        let _ = result_tx.send(result);
    });

    // Run the splash window on the main thread (required by macOS)
    window::show_splash(pixels, width, height, done);

    // Return the work result
    result_rx.recv().expect("bootstrap thread completed")
}

#[cfg(not(feature = "splash"))]
pub fn run_with_splash<F, R>(work: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    work()
}

#[cfg(feature = "splash")]
pub fn is_enabled() -> bool {
    env!("PYAPP_SPLASH_ENABLED") == "1"
}

#[cfg(not(feature = "splash"))]
pub fn is_enabled() -> bool {
    false
}

// Legacy no-op stubs retained for call sites pending removal in a follow-up task.
pub fn update(_status: &str, _progress: f32) {}
pub fn update_status(_status: &str) {}
pub fn close() {}
