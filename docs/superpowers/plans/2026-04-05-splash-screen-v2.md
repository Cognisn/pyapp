# Splash Screen v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the GPU-dependent eframe/egui splash screen with a static image splash using winit+softbuffer that works on all platforms including VMs without GPU.

**Architecture:** User provides a PNG/JPEG image at build time which is embedded into the binary. At runtime, the image is decoded, displayed in a borderless window via software rendering (winit+softbuffer), and closed when bootstrap completes. No GPU required. Splash runs on main thread (macOS requirement), bootstrap on background thread.

**Tech Stack:** Rust, winit 0.30, softbuffer 0.4, image 0.25

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Rewrite | `src/splash/mod.rs` | Public API: `run_with_splash()`, `is_enabled()`; no-op stubs when feature disabled |
| Rewrite | `src/splash/window.rs` | winit+softbuffer window creation, image decoding, pixel blitting |
| Delete | `src/splash/config.rs` | No longer needed (no theme/colour config) |
| Delete | `src/splash/progress.rs` | No longer needed (no progress updates) |
| Modify | `Cargo.toml` | Replace eframe/egui with winit/softbuffer; update feature flag |
| Modify | `build.rs:1291-1393` | Simplify `set_splash()` — require image, remove theme/colour/dimension vars |
| Modify | `src/distribution.rs` | Remove all `splash::update()` and `splash::update_status()` calls |
| Modify | `src/network.rs` | Remove splash imports and progress reporting |
| Rewrite | `docs/config/splash.md` | Updated documentation |
| Keep | `src/splash/embedded_logo.bin` | Still used for `include_bytes!` |
| Keep | `src/main.rs` | `mod splash;` already present |

---

### Task 1: Update Cargo Dependencies

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Replace eframe/egui with winit/softbuffer**

In `Cargo.toml`, replace lines 26-28:

```toml
eframe = { version = "0.29", optional = true, default-features = false, features = ["default_fonts", "wgpu", "glow"] }
egui = { version = "0.29", optional = true }
image = { version = "0.25", optional = true, default-features = false, features = ["png", "jpeg"] }
```

With:

```toml
winit = { version = "0.30", optional = true }
softbuffer = { version = "0.4", optional = true }
image = { version = "0.25", optional = true, default-features = false, features = ["png", "jpeg"] }
```

- [ ] **Step 2: Update feature flag**

Replace line 42:

```toml
splash = ["eframe", "egui", "image"]
```

With:

```toml
splash = ["winit", "softbuffer", "image"]
```

- [ ] **Step 3: Remove old splash passthrough vars**

In the `passthrough` array, replace lines 97-105:

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

With:

```toml
  "PYAPP_SPLASH_ENABLED",
  "PYAPP_SPLASH_IMAGE",
```

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "feat(splash-v2): replace eframe/egui with winit/softbuffer dependencies"
```

---

### Task 2: Simplify build.rs

**Files:**
- Modify: `build.rs:1291-1393`

- [ ] **Step 1: Replace the entire `set_splash()` function**

Replace the `set_splash()` function (lines 1291-1393) with:

```rust
fn set_splash() {
    let variable = "PYAPP_SPLASH_ENABLED";
    if is_enabled(variable) {
        set_runtime_variable(variable, "1");
        println!("cargo:rustc-cfg=feature=\"splash\"");

        // Splash image is required when splash is enabled
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let logo_dest = Path::new(&manifest_dir).join("src/splash/embedded_logo.bin");

        let image_path = env::var("PYAPP_SPLASH_IMAGE").unwrap_or_default();
        if image_path.is_empty() {
            panic!(
                "\n\nPYAPP_SPLASH_IMAGE is required when PYAPP_SPLASH_ENABLED is true.\n\
                 Provide a path to a PNG or JPEG image (recommended size: 640x400).\n\n"
            );
        }

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
        set_runtime_variable(variable, "0");

        // Ensure embedded_logo.bin exists even when splash is disabled,
        // so include_bytes! compiles if the feature is enabled via
        // --features splash without the PYAPP_SPLASH_ENABLED env var.
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let logo_dest = Path::new(&manifest_dir).join("src/splash/embedded_logo.bin");
        if logo_dest.parent().map_or(false, |p| p.is_dir()) {
            let _ = fs::write(&logo_dest, b"");
        }
    }
}
```

- [ ] **Step 2: Verify build**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles (splash module will have errors but default build should work)

- [ ] **Step 3: Commit**

```bash
git add build.rs
git commit -m "feat(splash-v2): simplify build.rs — require image, remove theme/colour config"
```

---

### Task 3: Rewrite splash window module

**Files:**
- Rewrite: `src/splash/window.rs`

- [ ] **Step 1: Replace window.rs with winit+softbuffer implementation**

Replace the entire content of `src/splash/window.rs` with:

```rust
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

pub struct SplashWindow {
    pixels: Vec<u32>,
    width: u32,
    height: u32,
    done: Arc<AtomicBool>,
    window: Option<Window>,
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

        let size = self.window.as_ref().unwrap().inner_size();
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
        self.window = Some(Arc::try_unwrap(window).unwrap_or_else(|arc| (*arc).clone()));

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
                // Schedule next check
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CloseRequested => {
                // Don't exit — bootstrap is still running.
                // Just ignore the close request.
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
```

- [ ] **Step 2: Commit**

```bash
git add src/splash/window.rs
git commit -m "feat(splash-v2): rewrite window module with winit+softbuffer"
```

---

### Task 4: Rewrite splash public API module

**Files:**
- Rewrite: `src/splash/mod.rs`
- Delete: `src/splash/config.rs`
- Delete: `src/splash/progress.rs`

- [ ] **Step 1: Replace mod.rs with simplified API**

Replace the entire content of `src/splash/mod.rs` with:

```rust
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
```

- [ ] **Step 2: Delete config.rs and progress.rs**

```bash
rm src/splash/config.rs src/splash/progress.rs
```

- [ ] **Step 3: Verify default build compiles**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src/splash/mod.rs
git rm src/splash/config.rs src/splash/progress.rs
git commit -m "feat(splash-v2): rewrite mod.rs with simplified API, delete config and progress modules"
```

---

### Task 5: Remove splash update calls from distribution.rs and network.rs

**Files:**
- Modify: `src/distribution.rs`
- Modify: `src/network.rs`

- [ ] **Step 1: Remove all `splash::update()` and `splash::update_status()` calls from distribution.rs**

Remove these lines from `src/distribution.rs`:

- Line 213: `splash::update("Checking distribution cache...", 0.0);`
- Line 235: `splash::update("Extracting embedded distribution...", 0.05);`
- Line 243: `splash::update("Downloading Python distribution...", 0.05);`
- Line 250: `splash::update("Unpacking distribution...", 0.30);`
- Line 270: `splash::update("Distribution ready", 0.40);`
- Line 338: `splash::update("Virtual environment created", 0.50);`
- Line 349: `splash::update(&format!("Installing {}...", app::project_name()), 0.60);`
- Line 480: `splash::update("Downloading pip...", 0.55);`
- Line 518: `splash::update("Downloading UV package manager...", 0.55);`

Also remove the `use crate::splash;` import from distribution.rs — but keep it because `splash::is_enabled()` and `splash::run_with_splash()` are still used in `ensure_ready()`.

- [ ] **Step 2: Revert network.rs to original (remove splash integration)**

Replace the entire content of `src/network.rs` with the original upstream version:

```rust
use std::io::Write;

use anyhow::{bail, Context, Result};

use crate::terminal;

pub fn download(url: &String, writer: impl Write, description: &str) -> Result<()> {
    let mut response =
        reqwest::blocking::get(url).with_context(|| format!("download failed: {}", url))?;

    let pb = terminal::io_progress_bar(
        format!("Downloading {}", description),
        response.content_length().unwrap_or(0),
    );
    response.copy_to(&mut pb.wrap_write(writer))?;
    pb.finish_and_clear();

    if response.status().is_success() {
        Ok(())
    } else {
        bail!("download failed: {}, {}", response.status(), url)
    }
}
```

- [ ] **Step 3: Verify build**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src/distribution.rs src/network.rs
git commit -m "feat(splash-v2): remove splash progress update calls from bootstrap flow"
```

---

### Task 6: Update documentation

**Files:**
- Rewrite: `docs/config/splash.md`
- Modify: `docs/build.md`
- Modify: `docs/runtime.md`
- Modify: `mkdocs.yml` (no changes needed — already has splash entry)

- [ ] **Step 1: Rewrite splash documentation**

Replace the entire content of `docs/config/splash.md` with:

```markdown
# Splash screen configuration

-----

The splash screen displays a static image during the first-run bootstrap process, providing visual feedback while the Python distribution is downloaded and your application's dependencies are installed. It appears automatically on first run and during `self restore`, and closes when bootstrap completes.

!!! note
    The splash screen is part of the [Cognisn fork](https://github.com/cognisn/pyapp) of PyApp. It is not available in upstream PyApp.

## Enabling

To enable the splash screen, you must:

1. Set the `PYAPP_SPLASH_ENABLED` environment variable to `true` or `1`
2. Set the `PYAPP_SPLASH_IMAGE` environment variable to the path of your splash image
3. Build with the `splash` Cargo feature: `cargo build --release --features splash`

The image is embedded into the binary at build time. No GPU is required — the splash uses software rendering that works on all platforms, including virtual machines and RDP sessions.

When not enabled, no splash dependencies are included and the binary is identical to upstream PyApp.

## Image

Set `PYAPP_SPLASH_IMAGE` to the path of a PNG or JPEG image file. This variable is **required** when splash is enabled — the build will fail without it.

The splash window is borderless (no title bar) and sizes itself to match the image exactly. No scaling is applied.

### Recommended dimensions

**640x400 pixels** — large enough to be clearly visible, small enough for all screen sizes including 1366x768 laptops.

### Design guidelines

Since the splash screen displays your image as-is with no overlaid text or progress bar, design your image to include:

- Your application logo or branding
- A message such as "Setting up, please wait..." or "Preparing first launch..."
- Any other visual elements you want the user to see during setup

## Configuration summary

| Variable | Required | Description |
|----------|----------|-------------|
| `PYAPP_SPLASH_ENABLED` | No | Set to `true` or `1` to enable |
| `PYAPP_SPLASH_IMAGE` | Yes (when enabled) | Path to PNG or JPEG image file |

## Example

=== "Linux/macOS"

    ```bash
    export PYAPP_PROJECT_NAME="myapp"
    export PYAPP_PROJECT_VERSION="1.0.0"
    export PYAPP_EXEC_MODULE="myapp"

    # Enable splash screen
    export PYAPP_SPLASH_ENABLED="true"
    export PYAPP_SPLASH_IMAGE="/path/to/splash.png"

    cargo build --release --features splash
    ```

=== "Windows (PowerShell)"

    ```powershell
    $env:PYAPP_PROJECT_NAME = "myapp"
    $env:PYAPP_PROJECT_VERSION = "1.0.0"
    $env:PYAPP_EXEC_MODULE = "myapp"

    # Enable splash screen
    $env:PYAPP_SPLASH_ENABLED = "true"
    $env:PYAPP_SPLASH_IMAGE = "C:\path\to\splash.png"

    cargo build --release --features splash
    ```

## Behaviour

- The splash screen only appears during first-run bootstrap and `self restore`. Subsequent runs skip it entirely.
- If the splash window fails to create (e.g. on a headless server with no display), bootstrap proceeds silently.
- The splash window runs on a separate thread and never blocks the bootstrap process.
- The window close button is ignored — closing the splash window does not cancel the installation.
- No user input is accepted through the splash window; it is display-only.
- No GPU is required. The splash uses software rendering via `softbuffer`, which works on all platforms including virtual machines, RDP sessions, and environments without GPU drivers.

## Cross-compilation

When using [cross](https://github.com/cross-rs/cross), the `PYAPP_SPLASH_ENABLED` and `PYAPP_SPLASH_IMAGE` environment variables are passed through to the build container automatically.

Platform-specific windowing dependencies may be required:

- **Linux:** `libx11-dev`, `libxcb-dev` (or equivalents for Wayland)
- **macOS:** Xcode Command Line Tools
- **Windows:** No additional dependencies
```

- [ ] **Step 2: Verify build.md and runtime.md are still correct**

The existing mentions in `docs/build.md` and `docs/runtime.md` reference the splash screen generically and still apply. No changes needed.

- [ ] **Step 3: Commit**

```bash
git add docs/config/splash.md
git commit -m "docs: rewrite splash screen docs for v2 static image approach"
```

---

### Task 7: Create Claude Code instruction document

**Files:**
- Create: `docs/cognisn-pyapp-fork.md`

- [ ] **Step 1: Create the instruction document**

Create `docs/cognisn-pyapp-fork.md`:

```markdown
# Cognisn PyApp Fork — Integration Guide

This document describes the Cognisn fork of [PyApp](https://github.com/ofek/pyapp) and how to use it as an alternative to the upstream project. It is suitable for inclusion in project instructions (e.g. CLAUDE.md) for AI coding assistants.

## Overview

[Cognisn/pyapp](https://github.com/Cognisn/pyapp) is a maintained fork of `ofek/pyapp` (a Rust-based runtime installer/bootstrapper for Python applications). The fork adds an optional splash screen that displays during the first-run bootstrap process.

**Upstream:** `ofek/pyapp` (Apache-2.0 OR MIT)
**Fork:** `Cognisn/pyapp`

## Key Differences from Upstream

- **Extra Cargo feature:** `splash` (optional, adds ~650KB to binary)
- **Extra env vars:** `PYAPP_SPLASH_ENABLED`, `PYAPP_SPLASH_IMAGE`
- **Versioning:** `v{upstream}-splash.{patch}` (e.g. `v0.29.0-splash.4`)
- **Splash code:** isolated in `src/splash/` to minimise merge conflicts with upstream

## Splash Screen

The splash screen displays a user-provided static image in a borderless window during first-run bootstrap. It uses software rendering (winit + softbuffer) — no GPU required. Works on all platforms including Windows VMs, RDP sessions, and headless environments (where it silently falls back to no splash).

### Configuration

| Variable | Required | Description |
|----------|----------|-------------|
| `PYAPP_SPLASH_ENABLED` | No | Set to `true` or `1` to enable |
| `PYAPP_SPLASH_IMAGE` | Yes (when enabled) | Path to PNG or JPEG image (recommended: 640x400px) |

### Building

```bash
# Clone the Cognisn fork (not upstream)
git clone https://github.com/Cognisn/pyapp.git
cd pyapp
git checkout v0.29.0-splash.4  # or latest tag

# Set project configuration
export PYAPP_PROJECT_NAME="<project>"
export PYAPP_PROJECT_VERSION="<version>"
export PYAPP_EXEC_MODULE="<module>"

# Enable splash screen
export PYAPP_SPLASH_ENABLED="true"
export PYAPP_SPLASH_IMAGE="/path/to/splash.png"

# Build with splash feature
cargo build --release --features splash
```

To build **without** the splash screen, omit `PYAPP_SPLASH_ENABLED` and `--features splash`. The binary will be identical to upstream PyApp.

### Image Design

The splash displays your image as-is — no overlaid text, progress bar, or UI elements. Design your image (recommended 640x400px PNG) to include:

- Your application logo/branding
- A message like "Setting up, please wait..."
- Any visual elements for the first-run experience

### Hatch Integration

If using Hatch's `app` build target, override the PyApp source URL:

```bash
export PYAPP_SOURCE="https://github.com/Cognisn/pyapp/archive/refs/tags/v0.29.0-splash.4.tar.gz"
```

## Upstream Sync

The fork tracks upstream `ofek/pyapp` (maintenance mode, ~2-3 month release cadence). Splash code is isolated in `src/splash/` to minimise merge conflicts.

```bash
git fetch upstream
git merge upstream/master
cargo test
cargo build --release --features splash
```

## Documentation

- [Splash screen configuration](https://github.com/Cognisn/pyapp/blob/master/docs/config/splash.md)
- [PyApp documentation](https://ofek.dev/pyapp/latest/)
```

- [ ] **Step 2: Commit**

```bash
git add docs/cognisn-pyapp-fork.md
git commit -m "docs: add Cognisn PyApp fork integration guide for AI assistants"
```

---

### Task 8: Update test app and verify builds

**Files:**
- Modify: `test-splash-app/build.sh`

- [ ] **Step 1: Update test build script**

Replace the content of `test-splash-app/build.sh` with:

```bash
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PYAPP_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building PyApp with splash screen enabled..."
echo

# Project configuration — uses cowsay as a small real PyPI package
export PYAPP_PROJECT_NAME="cowsay"
export PYAPP_PROJECT_VERSION="6.1"
export PYAPP_EXEC_MODULE="cowsay"

# Splash screen configuration
export PYAPP_SPLASH_ENABLED="true"
export PYAPP_SPLASH_IMAGE="${SPLASH_IMAGE:-$SCRIPT_DIR/splash.png}"

if [ ! -f "$PYAPP_SPLASH_IMAGE" ]; then
    echo "ERROR: Splash image not found: $PYAPP_SPLASH_IMAGE"
    echo "Create a 640x400 PNG splash image or set SPLASH_IMAGE=/path/to/image.png"
    exit 1
fi

echo "Configuration:"
echo "  Project:  $PYAPP_PROJECT_NAME v$PYAPP_PROJECT_VERSION (from PyPI)"
echo "  Image:    $PYAPP_SPLASH_IMAGE"
echo

cd "$PYAPP_DIR"
cargo build --release --features splash

BINARY="$PYAPP_DIR/target/release/pyapp"
echo
echo "Build complete!"
echo "Binary: $BINARY"
echo
echo "To test the splash screen:"
echo "  1. First run (shows splash):  $BINARY Hello splash screen!"
echo "  2. Second run (no splash):    $BINARY Moo"
echo "  3. Re-test (shows splash):    $BINARY self restore"
```

- [ ] **Step 2: Run cargo check (default build, no splash)**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles successfully

- [ ] **Step 3: Run cargo check (with splash feature)**

Run: `PYAPP_PROJECT_NAME=cowsay PYAPP_PROJECT_VERSION=6.1 PYAPP_EXEC_MODULE=cowsay PYAPP_SPLASH_ENABLED=true PYAPP_SPLASH_IMAGE=/dev/null cargo check --features splash 2>&1 | tail -10`

Note: Using `/dev/null` as image path — build.rs will accept it as a valid file. The binary won't display a splash (empty image) but compilation is verified.

Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add test-splash-app/build.sh
git commit -m "chore: update test build script for splash v2"
```

---

### Task 9: Version bump and final verification

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Bump version**

Change version in `Cargo.toml` from `0.29.0-splash.3` to `0.29.0-splash.4`.

- [ ] **Step 2: Run full cargo check (default)**

Run: `cargo check 2>&1 | tail -5`
Expected: Compiles with no errors

- [ ] **Step 3: Run cargo build release (default)**

Run: `cargo build --release 2>&1 | tail -5`
Expected: Builds successfully

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "chore: bump version to v0.29.0-splash.4"
```
