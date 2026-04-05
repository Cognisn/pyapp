# PyApp Splash Screen v2 - Static Image Design

## Overview

Replace the eframe/egui GPU-rendered splash screen with a static image splash using software rendering. The user provides a PNG/JPEG image at build time that is displayed as-is in a borderless window during first-run bootstrap and `self restore`. No GPU required.

**Problem with v1:** The eframe/egui approach requires GPU rendering (OpenGL, Direct3D, or Vulkan). This fails on Hyper-V VMs, RDP sessions, and environments without GPU adapters — exactly the kind of environments where PyApp-wrapped applications are commonly deployed.

**Solution:** Display a user-provided static image using `winit` (cross-platform windowing) + `softbuffer` (CPU-based pixel blitting). This works everywhere — no GPU, no OpenGL, no Direct3D. The user designs their splash image to include branding, messaging ("Setting up, please wait..."), and any visual elements they want.

## Architecture

### Dependencies

| Crate | Purpose | Size Impact |
|-------|---------|-------------|
| `winit` | Cross-platform window creation | ~300KB |
| `softbuffer` | Software pixel blitting to window | ~50KB |
| `image` | PNG/JPEG decoding | ~300KB |

**Removed:** `eframe` (~1MB), `egui` (included with eframe)

Total estimated binary size increase: ~650KB (down from ~1.5MB).

### Module Structure

```
src/splash/
  mod.rs          - Public API: run_with_splash(), is_enabled()
  window.rs       - winit + softbuffer window creation and image display
```

**Removed:** `config.rs`, `progress.rs` (no longer needed — no progress bar, no theme config)

### Threading Model

```
Main Thread                    Background Thread
-----------                    -----------------
run_with_splash(work) ──┐
                        ├───> Spawn: work()
Create window           │         │
Decode image            │     bootstrap runs...
Paint image to window   │         │
Enter event loop ◄──────┘     bootstrap completes
  │                           send Close signal
  ├── receive Close ──────> Exit event loop
  │
Return work result
```

Same as v1: splash on main thread (macOS requires this), bootstrap on background thread. Communication via `Arc<AtomicBool>` flag (simpler than mpsc since there are no status updates — just "done").

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PYAPP_SPLASH_ENABLED` | No | `false` | Set to `true` or `1` to enable |
| `PYAPP_SPLASH_IMAGE` | Yes (when enabled) | None | Path to PNG or JPEG image file |

**Removed variables:** `PYAPP_SPLASH_THEME`, `PYAPP_SPLASH_BG_COLOR`, `PYAPP_SPLASH_TEXT_COLOR`, `PYAPP_SPLASH_PROGRESS_COLOR`, `PYAPP_SPLASH_WINDOW_TITLE`, `PYAPP_SPLASH_WINDOW_WIDTH`, `PYAPP_SPLASH_WINDOW_HEIGHT`

### Build-time validation

When `PYAPP_SPLASH_ENABLED=true`:
- `PYAPP_SPLASH_IMAGE` must be set and point to a valid file
- Build fails with clear error if missing: `PYAPP_SPLASH_IMAGE is required when PYAPP_SPLASH_ENABLED is true`
- Image is copied to `src/splash/embedded_logo.bin` for `include_bytes!`
- `cargo:rerun-if-changed` tracks the image file

### Recommended image dimensions

**640x400 pixels** — large enough to be clearly visible, small enough for all screen sizes (including 1366x768 laptops). 16:10 aspect ratio.

The window sizes itself to match the image exactly. No scaling is applied.

## Runtime Behaviour

1. Check if installation directory exists — if yes, skip splash entirely
2. Decode embedded image bytes to RGBA pixel buffer
3. Create a borderless window (no title bar, no resize handles) sized to image dimensions
4. Center window on screen
5. Set window always-on-top
6. Paint image pixels to window surface via softbuffer
7. Spawn bootstrap work on background thread
8. Enter winit event loop (handles window repaints)
9. When bootstrap completes, set `done` flag
10. Event loop detects flag, closes window, returns

### Graceful degradation

- If window creation fails (headless, no display server): bootstrap proceeds silently
- If image decoding fails: bootstrap proceeds silently
- The splash is purely cosmetic; no bootstrap logic depends on it

### When splash appears

- First run (installation directory does not exist)
- `self restore` (removes installation, then re-bootstraps via `ensure_ready()`)

## Build Integration

### Cargo.toml changes

```toml
[features]
splash = ["winit", "softbuffer", "image"]

[dependencies]
winit = { version = "0.30", optional = true }
softbuffer = { version = "0.4", optional = true }
image = { version = "0.25", optional = true, default-features = false, features = ["png", "jpeg"] }
```

Remove: `eframe`, `egui`

### build.rs changes

Simplify `set_splash()` to only handle `PYAPP_SPLASH_ENABLED` and `PYAPP_SPLASH_IMAGE`. Remove all theme/colour/dimension handling. Require image when enabled.

### Cross-build passthrough

Reduce to two variables:
```toml
"PYAPP_SPLASH_ENABLED",
"PYAPP_SPLASH_IMAGE",
```

Remove: `PYAPP_SPLASH_BG_COLOR`, `PYAPP_SPLASH_TEXT_COLOR`, `PYAPP_SPLASH_PROGRESS_COLOR`, `PYAPP_SPLASH_THEME`, `PYAPP_SPLASH_WINDOW_HEIGHT`, `PYAPP_SPLASH_WINDOW_TITLE`, `PYAPP_SPLASH_WINDOW_WIDTH`

## Files Changed

| Action | File |
|--------|------|
| Rewrite | `src/splash/mod.rs` — simplified API: `run_with_splash()`, `is_enabled()` |
| Rewrite | `src/splash/window.rs` — winit + softbuffer implementation |
| Delete | `src/splash/config.rs` — no longer needed |
| Delete | `src/splash/progress.rs` — no longer needed |
| Modify | `Cargo.toml` — swap eframe/egui for winit/softbuffer |
| Modify | `build.rs` — simplify set_splash(), require image |
| Modify | `src/distribution.rs` — remove splash::update/update_status/close calls |
| Modify | `src/network.rs` — remove splash::update_status calls |
| Rewrite | `docs/config/splash.md` — updated documentation |
| Keep | `src/splash/embedded_logo.bin` — still used for include_bytes! |
| Keep | `.gitignore` entry — still ignores embedded_logo.bin |

## Testing

### Manual testing

| Scenario | Expected |
|----------|----------|
| First run with splash + image | Borderless window shows image, closes when bootstrap done |
| Subsequent run | No splash |
| `self restore` | Splash appears again |
| Headless/no display | Bootstrap proceeds silently |
| Ctrl+C during bootstrap | Process exits, window closes |
| `PYAPP_SPLASH_ENABLED=true` without image | Build fails with clear error |
| Large image (e.g. 1920x1080) | Window sized to image, may extend off small screens |
| Small image (e.g. 200x100) | Window sized to image, centered on screen |
