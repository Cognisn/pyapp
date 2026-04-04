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

#[cfg(feature = "splash")]
pub fn start() {
    let config = SplashConfig::from_env();
    let (sender, receiver) = std::sync::mpsc::channel();
    let handle = SplashHandle::new(sender);

    let window_title = config.window_title.clone();
    let window_width = config.window_width as f32;
    let window_height = config.window_height as f32;

    let _ = SPLASH_HANDLE.set(Mutex::new(handle));

    thread::spawn(move || {
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([window_width, window_height])
                .with_resizable(false)
                .with_decorations(true)
                .with_title(&window_title)
                .with_always_on_top(),
            ..Default::default()
        };

        let _ = eframe::run_native(
            &window_title,
            options,
            Box::new(move |_cc| Ok(Box::new(window::SplashApp::new(config, receiver)))),
        );
    });
}

#[cfg(not(feature = "splash"))]
pub fn start() {}

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
