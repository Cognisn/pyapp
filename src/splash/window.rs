use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

pub struct SplashWindow {
    pixels: Vec<u32>,
    width: u32,
    height: u32,
    done: Arc<AtomicBool>,
    window: Option<Arc<Window>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
}

impl SplashWindow {
    pub fn new(pixels: Vec<u32>, width: u32, height: u32, done: Arc<AtomicBool>) -> SplashWindow {
        SplashWindow {
            pixels,
            width,
            height,
            done,
            window: None,
            surface: None,
        }
    }

    fn paint(&mut self) {
        let Some(ref mut surface) = self.surface else {
            return;
        };

        let Some(ref window) = self.window else {
            return;
        };

        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        let Ok(mut buffer) = surface.buffer_mut() else {
            return;
        };

        let buf_width = size.width as usize;
        let buf_height = size.height as usize;
        let img_width = self.width as usize;
        let img_height = self.height as usize;

        for y in 0..buf_height {
            for x in 0..buf_width {
                let idx = y * buf_width + x;
                if x < img_width && y < img_height {
                    buffer[idx] = self.pixels[y * img_width + x];
                } else {
                    buffer[idx] = 0;
                }
            }
        }

        let _ = buffer.present();
    }
}

impl ApplicationHandler for SplashWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_inner_size(LogicalSize::new(self.width, self.height))
            .with_resizable(false)
            .with_decorations(false)
            .with_title("Loading...");

        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("Splash: failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let context = match softbuffer::Context::new(window.clone()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Splash: failed to create softbuffer context: {}", e);
                event_loop.exit();
                return;
            }
        };

        let mut surface = match softbuffer::Surface::new(&context, window.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Splash: failed to create surface: {}", e);
                event_loop.exit();
                return;
            }
        };

        let _ = surface.resize(
            NonZeroU32::new(self.width).unwrap_or(NonZeroU32::new(1).unwrap()),
            NonZeroU32::new(self.height).unwrap_or(NonZeroU32::new(1).unwrap()),
        );

        self.surface = Some(surface);
        self.window = Some(window);

        self.paint();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                if self.done.load(Ordering::Relaxed) {
                    event_loop.exit();
                    return;
                }
                self.paint();
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CloseRequested => {
                // Don't exit — bootstrap is still running.
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.done.load(Ordering::Relaxed) {
            event_loop.exit();
            return;
        }
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

pub fn show_splash(pixels: Vec<u32>, width: u32, height: u32, done: Arc<AtomicBool>) {
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            eprintln!("Splash: failed to create event loop: {}", e);
            return;
        }
    };

    let mut app = SplashWindow::new(pixels, width, height, done);
    let _ = event_loop.run_app(&mut app);
}
