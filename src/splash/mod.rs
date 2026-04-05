#[cfg(feature = "splash")]
mod config;
#[cfg(feature = "splash")]
mod progress;
#[cfg(feature = "splash")]
mod window;

#[cfg(feature = "splash")]
use std::sync::Mutex;
#[cfg(feature = "splash")]
use std::thread;

#[cfg(feature = "splash")]
use once_cell::sync::OnceCell;

#[cfg(feature = "splash")]
use self::config::SplashConfig;
#[cfg(feature = "splash")]
use self::progress::SplashHandle;

#[cfg(feature = "splash")]
static SPLASH_HANDLE: OnceCell<Mutex<SplashHandle>> = OnceCell::new();

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
    use std::sync::mpsc as std_mpsc;

    let config = SplashConfig::from_env();
    let (sender, receiver) = std::sync::mpsc::channel();
    let handle = SplashHandle::new(sender);
    let _ = SPLASH_HANDLE.set(Mutex::new(handle));

    let window_title = config.window_title.clone();
    let window_width = config.window_width as f32;
    let window_height = config.window_height as f32;

    // Channel to get the work result back from the background thread
    let (result_tx, result_rx) = std_mpsc::channel();

    // Spawn the bootstrap work on a background thread
    thread::spawn(move || {
        let result = work();
        // Signal splash to close (ignore error if splash already closed)
        if let Some(handle) = SPLASH_HANDLE.get() {
            if let Ok(h) = handle.lock() {
                h.close();
            }
        }
        let _ = result_tx.send(result);
    });

    // Run the splash window on the main thread (required by macOS).
    // Try wgpu first (Direct3D/Metal/Vulkan), fall back to glow (OpenGL).
    let viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([window_width, window_height])
        .with_resizable(false)
        .with_decorations(true)
        .with_title(&window_title)
        .with_always_on_top();

    let wgpu_options = eframe::NativeOptions {
        viewport: viewport.clone(),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    let splash_started = match eframe::run_native(
        &window_title,
        wgpu_options,
        Box::new(move |_cc| Ok(Box::new(window::SplashApp::new(config, receiver)))),
    ) {
        Ok(()) => true,
        Err(e) => {
            eprintln!("Splash wgpu renderer failed: {}, trying glow...", e);
            false
        }
    };

    if !splash_started {
        // wgpu failed — try glow (OpenGL) as fallback.
        // Need fresh config and channel since the previous ones were moved.
        let config2 = SplashConfig::from_env();
        let (sender2, receiver2) = std::sync::mpsc::channel();
        // Replace the handle so update/close calls go to the new channel
        if let Some(handle) = SPLASH_HANDLE.get() {
            if let Ok(mut h) = handle.lock() {
                *h = SplashHandle::new(sender2);
            }
        }

        let glow_options = eframe::NativeOptions {
            viewport,
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        };

        if let Err(e) = eframe::run_native(
            &window_title,
            glow_options,
            Box::new(move |_cc| Ok(Box::new(window::SplashApp::new(config2, receiver2)))),
        ) {
            eprintln!("Splash glow renderer also failed: {}", e);
        }
    }

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
pub fn update(status: &str, progress: f32) {
    if let Some(handle) = SPLASH_HANDLE.get() {
        if let Ok(h) = handle.lock() {
            h.update_status(status);
            h.update_progress(progress);
        }
    }
}

#[cfg(not(feature = "splash"))]
pub fn update(_status: &str, _progress: f32) {}

#[cfg(feature = "splash")]
pub fn update_status(status: &str) {
    if let Some(handle) = SPLASH_HANDLE.get() {
        if let Ok(h) = handle.lock() {
            h.update_status(status);
        }
    }
}

#[cfg(not(feature = "splash"))]
pub fn update_status(_status: &str) {}

#[cfg(feature = "splash")]
pub fn close() {
    if let Some(handle) = SPLASH_HANDLE.get() {
        if let Ok(h) = handle.lock() {
            h.close();
        }
    }
}

#[cfg(not(feature = "splash"))]
pub fn close() {}

#[cfg(feature = "splash")]
pub fn is_enabled() -> bool {
    env!("PYAPP_SPLASH_ENABLED") == "1"
}

#[cfg(not(feature = "splash"))]
pub fn is_enabled() -> bool {
    false
}
