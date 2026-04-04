# Splash Screen Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an optional cross-platform GUI splash screen that displays during PyApp's first-run bootstrap and `self restore`, showing a logo, progress bar, and status messages.

**Architecture:** A `src/splash/` module behind a `splash` Cargo feature flag uses `eframe`/`egui` for cross-platform GUI. The splash runs on a separate thread, communicating via `std::sync::mpsc` channels. All configuration is embedded at build time via environment variables processed in `build.rs`. When the feature is disabled, all splash functions compile to no-ops.

**Tech Stack:** Rust, eframe 0.29, egui 0.29, image 0.25 (PNG/JPEG decoding)

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `src/splash/mod.rs` | Public API (`start`, `update`, `close`) + no-op stubs when feature disabled |
| Create | `src/splash/config.rs` | `SplashConfig` struct parsed from compile-time env vars, hex colour parsing, theme defaults |
| Create | `src/splash/progress.rs` | `SplashMessage` enum, `SplashHandle` struct wrapping mpsc sender |
| Create | `src/splash/window.rs` | `SplashApp` implementing `eframe::App`, egui rendering (logo, progress bar, status text) |
| Modify | `Cargo.toml` | Add `splash` feature flag and optional dependencies |
| Modify | `build.rs:1291-1314` | Add `set_splash()` function call in `main()`, implement `set_splash()` |
| Modify | `src/main.rs:1-35` | Add `mod splash;` declaration |
| Modify | `src/distribution.rs:137-151` | Add splash start/update/close calls in `ensure_ready()` |
| Modify | `src/distribution.rs:195-320` | Add splash update calls in `materialize()` |
| Modify | `src/distribution.rs:322-375` | Add splash update calls in `install_project()` |
| Modify | `src/distribution.rs:418-469` | Add splash update calls in `ensure_installer_available()` |
| Modify | `src/network.rs:1-23` | Add splash progress callback for byte-level download progress |

---

### Task 1: Add Cargo Dependencies and Feature Flag

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add splash feature and optional dependencies to Cargo.toml**

Add after line 25 (`zstd = "0.13.2"`):

```toml
eframe = { version = "0.29", optional = true, default-features = false, features = ["default_fonts", "glow"] }
egui = { version = "0.29", optional = true }
image = { version = "0.25", optional = true, default-features = false, features = ["png", "jpeg"] }
```

Add after the `[build-dependencies]` section (after line 35):

```toml
[features]
default = []
splash = ["eframe", "egui", "image"]
```

- [ ] **Step 2: Add splash env vars to cross build passthrough**

Add to the `passthrough` array in `[package.metadata.cross.build.env]` (after the existing entries, before the closing `]`):

```toml
  "PYAPP_SPLASH_BG_COLOR",
  "PYAPP_SPLASH_ENABLED",
  "PYAPP_SPLASH_IMAGE",
  "PYAPP_SPLASH_PROGRESS_COLOR",
  "PYAPP_SPLASH_TEXT_COLOR",
  "PYAPP_SPLASH_THEME",
  "PYAPP_SPLASH_WINDOW_HEIGHT",
  "PYAPP_SPLASH_WINDOW_TITLE",
  "PYAPP_SPLASH_WINDOW_WIDTH",
```

- [ ] **Step 3: Verify it compiles without the splash feature**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles successfully (no splash code exists yet, feature just adds optional deps)

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "feat: add splash feature flag and optional eframe/egui/image dependencies"
```

---

### Task 2: Add build.rs Splash Configuration

**Files:**
- Modify: `build.rs:1291-1314`

- [ ] **Step 1: Add `set_splash()` function to build.rs**

Add before the `fn main()` function (before line 1291):

```rust
fn set_splash() {
    let variable = "PYAPP_SPLASH_ENABLED";
    if is_enabled(variable) {
        set_runtime_variable(variable, "1");

        // Enable the splash cargo feature
        println!("cargo:rustc-cfg=feature=\"splash\"");

        // Theme: "dark" (default) or "light"
        let theme = env::var("PYAPP_SPLASH_THEME").unwrap_or_default();
        let theme = if theme.eq_ignore_ascii_case("light") {
            "light"
        } else {
            "dark"
        };
        set_runtime_variable("PYAPP_SPLASH_THEME", theme);

        // Default colours based on theme
        let (default_bg, default_text, default_progress) = if theme == "light" {
            ("#f5f5f5", "#1a1a1a", "#2a6cb6")
        } else {
            ("#1a1a2e", "#ffffff", "#4a90d9")
        };

        let bg_color = env::var("PYAPP_SPLASH_BG_COLOR").unwrap_or_default();
        set_runtime_variable(
            "PYAPP_SPLASH_BG_COLOR",
            if bg_color.is_empty() { default_bg } else { &bg_color },
        );

        let text_color = env::var("PYAPP_SPLASH_TEXT_COLOR").unwrap_or_default();
        set_runtime_variable(
            "PYAPP_SPLASH_TEXT_COLOR",
            if text_color.is_empty() { default_text } else { &text_color },
        );

        let progress_color = env::var("PYAPP_SPLASH_PROGRESS_COLOR").unwrap_or_default();
        set_runtime_variable(
            "PYAPP_SPLASH_PROGRESS_COLOR",
            if progress_color.is_empty() { default_progress } else { &progress_color },
        );

        // Window title defaults to project name
        let window_title = env::var("PYAPP_SPLASH_WINDOW_TITLE").unwrap_or_default();
        if window_title.is_empty() {
            set_runtime_variable(
                "PYAPP_SPLASH_WINDOW_TITLE",
                env::var("PYAPP_PROJECT_NAME").unwrap_or_default(),
            );
        } else {
            set_runtime_variable("PYAPP_SPLASH_WINDOW_TITLE", &window_title);
        }

        // Window dimensions
        let width = env::var("PYAPP_SPLASH_WINDOW_WIDTH").unwrap_or_default();
        set_runtime_variable(
            "PYAPP_SPLASH_WINDOW_WIDTH",
            if width.is_empty() { "480" } else { &width },
        );

        let height = env::var("PYAPP_SPLASH_WINDOW_HEIGHT").unwrap_or_default();
        set_runtime_variable(
            "PYAPP_SPLASH_WINDOW_HEIGHT",
            if height.is_empty() { "360" } else { &height },
        );

        // Splash image (optional)
        if let Ok(image_path) = env::var("PYAPP_SPLASH_IMAGE") {
            if !image_path.is_empty() {
                let path = std::path::Path::new(&image_path);
                if !path.is_file() {
                    panic!(
                        "\n\nPYAPP_SPLASH_IMAGE is not a valid file: {}\n\n",
                        image_path
                    );
                }
                println!("cargo:rerun-if-changed={}", image_path);
                set_runtime_variable("PYAPP_SPLASH_IMAGE_PATH", &image_path);
            }
        }
    } else {
        set_runtime_variable(variable, "0");
    }
}
```

- [ ] **Step 2: Call `set_splash()` from `main()`**

In the `fn main()` function, add the `set_splash()` call before `set_skip_install()` (before line 1313 — the comment "This must come last"):

```rust
    set_splash();

    // This must come last because it might override a command exposure
    set_skip_install();
```

- [ ] **Step 3: Verify build.rs compiles**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add build.rs
git commit -m "feat: add splash screen env var handling in build.rs"
```

---

### Task 3: Create Splash Config Module

**Files:**
- Create: `src/splash/config.rs`

- [ ] **Step 1: Write tests for hex colour parsing**

Create `src/splash/config.rs` with the following content:

```rust
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn from_hex(hex: &str) -> Color {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color { r, g, b }
    }

    pub fn to_egui(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

pub struct SplashConfig {
    pub bg_color: Color,
    pub text_color: Color,
    pub progress_color: Color,
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub project_name: String,
    pub project_version: String,
}

impl SplashConfig {
    pub fn from_env() -> SplashConfig {
        SplashConfig {
            bg_color: Color::from_hex(env!("PYAPP_SPLASH_BG_COLOR")),
            text_color: Color::from_hex(env!("PYAPP_SPLASH_TEXT_COLOR")),
            progress_color: Color::from_hex(env!("PYAPP_SPLASH_PROGRESS_COLOR")),
            window_title: env!("PYAPP_SPLASH_WINDOW_TITLE").to_string(),
            window_width: env!("PYAPP_SPLASH_WINDOW_WIDTH")
                .parse()
                .unwrap_or(480),
            window_height: env!("PYAPP_SPLASH_WINDOW_HEIGHT")
                .parse()
                .unwrap_or(360),
            project_name: env!("PYAPP_PROJECT_NAME").to_string(),
            project_version: env!("PYAPP_PROJECT_VERSION").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_with_hash() {
        let color = Color::from_hex("#1a1a2e");
        assert_eq!(color.r, 0x1a);
        assert_eq!(color.g, 0x1a);
        assert_eq!(color.b, 0x2e);
    }

    #[test]
    fn test_parse_hex_color_without_hash() {
        let color = Color::from_hex("ffffff");
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 255);
    }

    #[test]
    fn test_parse_hex_color_black() {
        let color = Color::from_hex("#000000");
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_to_egui() {
        let color = Color::from_hex("#4a90d9");
        assert_eq!(color.to_egui(), [0x4a, 0x90, 0xd9]);
    }
}
```

- [ ] **Step 2: Run tests to verify colour parsing**

Run: `cargo test --features splash -- splash::config::tests 2>&1 | tail -10`

Note: This will fail until the module is wired up. We'll wire it up in Task 5. For now, the code is correct — we'll test after wiring.

- [ ] **Step 3: Commit**

```bash
git add src/splash/config.rs
git commit -m "feat: add splash config module with colour parsing and tests"
```

---

### Task 4: Create Splash Progress Module

**Files:**
- Create: `src/splash/progress.rs`

- [ ] **Step 1: Create the progress module with message types and handle**

Create `src/splash/progress.rs`:

```rust
use std::sync::mpsc;

pub enum SplashMessage {
    UpdateStatus(String),
    UpdateProgress(f32),
    Close,
}

pub struct SplashHandle {
    sender: mpsc::Sender<SplashMessage>,
}

impl SplashHandle {
    pub fn new(sender: mpsc::Sender<SplashMessage>) -> SplashHandle {
        SplashHandle { sender }
    }

    pub fn update_status(&self, status: &str) {
        let _ = self.sender.send(SplashMessage::UpdateStatus(status.to_string()));
    }

    pub fn update_progress(&self, progress: f32) {
        let _ = self.sender.send(SplashMessage::UpdateProgress(progress.clamp(0.0, 1.0)));
    }

    pub fn close(&self) {
        let _ = self.sender.send(SplashMessage::Close);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splash_message_channel() {
        let (tx, rx) = mpsc::channel();
        let handle = SplashHandle::new(tx);

        handle.update_status("Downloading Python...");
        handle.update_progress(0.5);
        handle.close();

        match rx.recv().unwrap() {
            SplashMessage::UpdateStatus(s) => assert_eq!(s, "Downloading Python..."),
            _ => panic!("expected UpdateStatus"),
        }
        match rx.recv().unwrap() {
            SplashMessage::UpdateProgress(p) => assert!((p - 0.5).abs() < f32::EPSILON),
            _ => panic!("expected UpdateProgress"),
        }
        match rx.recv().unwrap() {
            SplashMessage::Close => {}
            _ => panic!("expected Close"),
        }
    }

    #[test]
    fn test_progress_clamping() {
        let (tx, rx) = mpsc::channel();
        let handle = SplashHandle::new(tx);

        handle.update_progress(1.5);
        match rx.recv().unwrap() {
            SplashMessage::UpdateProgress(p) => assert!((p - 1.0).abs() < f32::EPSILON),
            _ => panic!("expected UpdateProgress"),
        }

        handle.update_progress(-0.5);
        match rx.recv().unwrap() {
            SplashMessage::UpdateProgress(p) => assert!(p.abs() < f32::EPSILON),
            _ => panic!("expected UpdateProgress"),
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/splash/progress.rs
git commit -m "feat: add splash progress module with message channel and handle"
```

---

### Task 5: Create Splash Window Module

**Files:**
- Create: `src/splash/window.rs`

- [ ] **Step 1: Create the eframe/egui window implementation**

Create `src/splash/window.rs`:

```rust
use std::sync::mpsc;

use eframe::egui;

use super::config::{Color, SplashConfig};
use super::progress::SplashMessage;

pub struct SplashApp {
    config: SplashConfig,
    receiver: mpsc::Receiver<SplashMessage>,
    status: String,
    progress: f32,
    logo_texture: Option<egui::TextureHandle>,
    logo_loaded: bool,
    should_close: bool,
}

impl SplashApp {
    pub fn new(config: SplashConfig, receiver: mpsc::Receiver<SplashMessage>) -> SplashApp {
        SplashApp {
            config,
            receiver,
            status: "Preparing...".to_string(),
            progress: 0.0,
            logo_texture: None,
            logo_loaded: false,
            should_close: false,
        }
    }

    fn process_messages(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                SplashMessage::UpdateStatus(s) => self.status = s,
                SplashMessage::UpdateProgress(p) => self.progress = p,
                SplashMessage::Close => self.should_close = true,
            }
        }
    }

    fn load_logo(&mut self, ctx: &egui::Context) {
        if self.logo_loaded {
            return;
        }
        self.logo_loaded = true;

        let image_bytes: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/splash/embedded_logo.bin"
        ));

        if image_bytes.is_empty() {
            return;
        }

        if let Ok(img) = image::load_from_memory(image_bytes) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = rgba.into_raw();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            self.logo_texture =
                Some(ctx.load_texture("splash_logo", color_image, egui::TextureOptions::LINEAR));
        }
    }

    fn color_to_egui32(color: &Color) -> egui::Color32 {
        egui::Color32::from_rgb(color.r, color.g, color.b)
    }
}

impl eframe::App for SplashApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_messages();

        if self.should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.load_logo(ctx);

        let bg = Self::color_to_egui32(&self.config.bg_color);
        let text_color = Self::color_to_egui32(&self.config.text_color);
        let progress_color = Self::color_to_egui32(&self.config.progress_color);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    // Logo image or fallback text
                    if let Some(ref texture) = self.logo_texture {
                        let mut size = texture.size_vec2();
                        let max_width = self.config.window_width as f32 * 0.6;
                        let max_height = self.config.window_height as f32 * 0.4;
                        let scale = (max_width / size.x).min(max_height / size.y).min(1.0);
                        size *= scale;
                        ui.image(egui::load::SizedTexture::new(texture.id(), size));
                    } else {
                        ui.add_space(40.0);
                        ui.label(
                            egui::RichText::new(&self.config.project_name)
                                .color(text_color)
                                .size(32.0)
                                .strong(),
                        );
                    }

                    ui.add_space(10.0);

                    // Version
                    if !self.config.project_version.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("v{}", self.config.project_version))
                                .color(text_color)
                                .size(14.0),
                        );
                    }

                    ui.add_space(20.0);

                    // Progress bar
                    let available_width = ui.available_width() * 0.8;
                    let bar_height = 20.0;
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(available_width, bar_height), egui::Sense::hover());

                    let painter = ui.painter();

                    // Background track
                    let track_color = egui::Color32::from_rgba_premultiplied(
                        text_color.r(),
                        text_color.g(),
                        text_color.b(),
                        30,
                    );
                    painter.rect_filled(rect, 4.0, track_color);

                    // Fill
                    let fill_width = rect.width() * self.progress;
                    let fill_rect = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(fill_width, rect.height()),
                    );
                    painter.rect_filled(fill_rect, 4.0, progress_color);

                    // Percentage text
                    let pct_text = format!("{}%", (self.progress * 100.0) as u32);
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        pct_text,
                        egui::FontId::proportional(12.0),
                        text_color,
                    );

                    ui.add_space(15.0);

                    // Status text
                    ui.label(
                        egui::RichText::new(&self.status)
                            .color(text_color)
                            .size(14.0),
                    );
                });
            });

        // Request repaint to stay responsive to messages
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }
}
```

- [ ] **Step 2: Create the embedded logo placeholder file**

Create an empty file `src/splash/embedded_logo.bin`. This file will be empty by default (no logo) and will be populated by `build.rs` when `PYAPP_SPLASH_IMAGE` is set.

```bash
touch src/splash/embedded_logo.bin
```

- [ ] **Step 3: Commit**

```bash
git add src/splash/window.rs src/splash/embedded_logo.bin
git commit -m "feat: add splash window module with eframe/egui rendering"
```

---

### Task 6: Create Splash Public API Module

**Files:**
- Create: `src/splash/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create the splash module public API**

Create `src/splash/mod.rs`:

```rust
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
```

- [ ] **Step 2: Add `mod splash;` to main.rs**

In `src/main.rs`, add `mod splash;` after the existing module declarations (after line 8, `mod terminal;`):

```rust
mod splash;
```

- [ ] **Step 3: Verify it compiles without the splash feature**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles successfully (no-op stubs used)

- [ ] **Step 4: Verify it compiles with the splash feature**

Run: `PYAPP_SPLASH_ENABLED=true cargo check --features splash 2>&1 | tail -10`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add src/splash/mod.rs src/main.rs
git commit -m "feat: add splash public API module with feature-gated start/update/close"
```

---

### Task 7: Integrate build.rs Image Embedding

**Files:**
- Modify: `build.rs`

The `set_splash()` function from Task 2 already reads `PYAPP_SPLASH_IMAGE` and sets `PYAPP_SPLASH_IMAGE_PATH`. Now we need to copy the image file to `src/splash/embedded_logo.bin` during the build so `include_bytes!` in `window.rs` picks it up.

- [ ] **Step 1: Update `set_splash()` to copy the image file**

In the `set_splash()` function in `build.rs`, replace the image handling block (the `if let Ok(image_path)` section) with:

```rust
        // Splash image (optional) — copy to embedded_logo.bin for include_bytes!
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let logo_dest = Path::new(&manifest_dir).join("src/splash/embedded_logo.bin");
        if let Ok(image_path) = env::var("PYAPP_SPLASH_IMAGE") {
            if !image_path.is_empty() {
                let path = Path::new(&image_path);
                if !path.is_file() {
                    panic!(
                        "\n\nPYAPP_SPLASH_IMAGE is not a valid file: {}\n\n",
                        image_path
                    );
                }
                println!("cargo:rerun-if-changed={}", image_path);
                fs::copy(&image_path, &logo_dest).unwrap_or_else(|e| {
                    panic!("unable to copy splash image to embedded_logo.bin: {}", e)
                });
            } else {
                // Write empty file (no logo)
                fs::write(&logo_dest, b"").unwrap();
            }
        } else {
            // Write empty file (no logo)
            fs::write(&logo_dest, b"").unwrap();
        }
```

Also remove the `set_runtime_variable("PYAPP_SPLASH_IMAGE_PATH", &image_path);` line — we no longer need it since we copy the file directly.

- [ ] **Step 2: Verify build still works**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add build.rs
git commit -m "feat: embed splash logo image into binary via build.rs file copy"
```

---

### Task 8: Integrate Splash into Bootstrap Flow

**Files:**
- Modify: `src/distribution.rs:137-151` (ensure_ready)
- Modify: `src/distribution.rs:195-320` (materialize)
- Modify: `src/distribution.rs:322-375` (install_project)
- Modify: `src/distribution.rs:418-469` (ensure_installer_available)

- [ ] **Step 1: Add splash import to distribution.rs**

Add to the `use` block at the top of `src/distribution.rs` (after line 12, `use crate::{app, compression, fs_utils, network, process};`):

```rust
use crate::splash;
```

- [ ] **Step 2: Add splash start and close in `ensure_ready()`**

Replace the `ensure_ready` function (lines 137-151) with:

```rust
pub fn ensure_ready() -> Result<()> {
    let lock_path = app::installation_lock();
    let lock_file = fs_utils::acquire_lock(&lock_path)?;

    if !app::install_dir().is_dir() {
        if splash::is_enabled() {
            splash::start();
        }

        materialize()?;

        if !app::skip_install() {
            install_project()?;
        }

        splash::close();
    }

    FileExt::unlock(&lock_file)
        .with_context(|| format!("unable to release lock file {}", lock_path.display()))
}
```

- [ ] **Step 3: Add splash updates in `materialize()`**

In `materialize()`, add splash update calls at each phase. Insert these calls at the appropriate points:

After line 198 (start of materialize, before the distribution file check):
```rust
    splash::update("Checking distribution cache...", 0.0);
```

Before line 219 (`if !app::embedded_distribution().is_empty()`), add:
```rust
        splash::update("Extracting embedded distribution...", 0.05);
```

Change the embedded distribution branch (line 219-224) — add a splash update before the `else` branch:

After line 225 (before `network::download`), add:
```rust
            splash::update("Downloading Python distribution...", 0.05);
```

After line 229 (`fs_utils::move_temp_file`), add:
```rust
    splash::update("Unpacking distribution...", 0.30);
```

After the full_isolation branch unpack (around line 246, after `ensure_base_pip`), add:
```rust
        splash::update("Distribution ready", 0.40);
```

In the non-full-isolation branch, after the venv creation `run_setup_command` (around line 315), add:
```rust
        splash::update("Virtual environment created", 0.50);
```

- [ ] **Step 4: Add splash updates in `install_project()`**

In `install_project()`, add a splash update at the start (after line 323):

```rust
    splash::update(&format!("Installing {}...", app::project_name()), 0.60);
```

- [ ] **Step 5: Add splash updates in `ensure_installer_available()`**

In `ensure_installer_available()`, add splash updates for downloading installers.

For pip download (after line 455, before `network::download` for pip), add:
```rust
        splash::update("Downloading pip...", 0.55);
```

For UV download (in `ensure_uv_available()`, before `network::download` for UV around line 493), add:
```rust
    splash::update("Downloading UV package manager...", 0.55);
```

- [ ] **Step 6: Verify it compiles without the splash feature**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles successfully (splash::update and splash::close are no-ops)

- [ ] **Step 7: Commit**

```bash
git add src/distribution.rs
git commit -m "feat: integrate splash screen updates into bootstrap flow"
```

---

### Task 9: Add Download Progress Reporting to Splash

**Files:**
- Modify: `src/network.rs`

- [ ] **Step 1: Add splash-aware download progress reporting**

Replace the contents of `src/network.rs` with:

```rust
use std::io::Write;

use anyhow::{bail, Context, Result};

use crate::splash;
use crate::terminal;

pub fn download(url: &String, writer: impl Write, description: &str) -> Result<()> {
    let mut response =
        reqwest::blocking::get(url).with_context(|| format!("download failed: {}", url))?;

    let total = response.content_length().unwrap_or(0);
    let pb = terminal::io_progress_bar(format!("Downloading {}", description), total);

    if splash::is_enabled() && total > 0 {
        // Wrap writer to report progress to splash as well
        let mut buf_writer = pb.wrap_write(writer);
        let mut downloaded: u64 = 0;
        let mut buf = [0u8; 8192];
        loop {
            let n = response.read(&mut buf).with_context(|| "download read failed")?;
            if n == 0 {
                break;
            }
            buf_writer.write_all(&buf[..n])?;
            downloaded += n as u64;
            // Map download progress to the 0.05-0.30 range (distribution download phase)
            let fraction = downloaded as f32 / total as f32;
            splash::update(
                &format!("Downloading {}... {}%", description, (fraction * 100.0) as u32),
                0.05 + fraction * 0.25,
            );
        }
    } else {
        response.copy_to(&mut pb.wrap_write(writer))?;
    }

    pb.finish_and_clear();

    if response.status().is_success() {
        Ok(())
    } else {
        bail!("download failed: {}, {}", response.status(), url)
    }
}
```

Add at the top of the file (after line 1):

```rust
use std::io::Read;
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src/network.rs
git commit -m "feat: report download progress to splash screen"
```

---

### Task 10: Run Full Test Suite and Verify Build

**Files:** None (verification only)

- [ ] **Step 1: Run cargo check without splash feature**

Run: `cargo check 2>&1 | tail -10`
Expected: Compiles successfully

- [ ] **Step 2: Run cargo check with splash feature**

Run: `PYAPP_SPLASH_ENABLED=true cargo check --features splash 2>&1 | tail -10`
Expected: Compiles successfully

- [ ] **Step 3: Run unit tests**

Run: `cargo test 2>&1 | tail -20`
Expected: All existing tests pass, splash config and progress tests pass

- [ ] **Step 4: Run unit tests with splash feature**

Run: `PYAPP_SPLASH_ENABLED=true cargo test --features splash 2>&1 | tail -20`
Expected: All tests pass including splash-specific tests

- [ ] **Step 5: Verify release build without splash**

Run: `cargo build --release 2>&1 | tail -5`
Expected: Builds successfully

- [ ] **Step 6: Commit any fixes if needed, then tag**

If all checks pass, no commit needed for this task.

---

### Task 11: Add Splash to Self Restore Command

**Files:**
- Modify: `src/commands/self_cmd/restore.rs`

The `self restore` command calls `distribution::ensure_ready()` which already has the splash integration from Task 8. Since `ensure_ready()` checks `!app::install_dir().is_dir()` and `restore` first calls `remove` (which deletes the install dir), the splash will automatically appear during restore.

- [ ] **Step 1: Verify restore already triggers splash**

Read `src/commands/self_cmd/restore.rs` and confirm the flow:
1. `super::remove::Cli {}.exec()?` — removes install directory
2. `distribution::ensure_ready()?` — checks `!app::install_dir().is_dir()` → true → starts splash

This already works. No code changes needed.

- [ ] **Step 2: Commit confirmation (no changes needed)**

No commit needed — the existing `ensure_ready()` integration handles this case.

---

### Task 12: Add .gitignore Entry for Embedded Logo

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: Add embedded_logo.bin to .gitignore**

The `src/splash/embedded_logo.bin` file is generated by `build.rs` and should not be tracked. Add to `.gitignore`:

```
src/splash/embedded_logo.bin
```

- [ ] **Step 2: Commit**

```bash
git add .gitignore
git commit -m "chore: ignore build-generated embedded_logo.bin"
```
