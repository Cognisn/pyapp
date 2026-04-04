# PyApp Splash Screen - Design Specification

## Overview

Add an optional, cross-platform GUI splash screen to the PyApp bootstrapper that displays during first-run setup and `self restore`. The splash screen shows a logo/text, progress bar, and status messages while Python is downloaded and dependencies are installed.

**Problem:** Users kill the process during first-run bootstrap because there's no visual feedback, corrupting the installation. (Upstream issue #140.)

## Architecture

### GUI Library

**eframe/egui** — provides cross-platform windowing with built-in text rendering, image display, and progress bar widgets from a single codebase. Adds ~1.5MB to binary size. Gated behind a `splash` Cargo feature flag so non-splash builds are identical to upstream.

### Module Structure

```
src/splash/
  mod.rs              - Public API: start(), update(), close(); no-op stubs when feature disabled
  config.rs           - Build-time configuration (theme, colours, dimensions, embedded assets)
  window.rs           - eframe/egui window creation and render loop
  progress.rs         - Thread-safe progress state and mpsc channel
```

### Threading Model

```
Main Thread                    Splash Thread
-----------                    -------------
start() ──────────────────────> Create window, enter render loop
bootstrap_phase()                     |
  |── update(status, pct) ───> Update display
  |── update(status, pct) ───> Update display
bootstrap_complete()                  |
  |── close() ───────────────> Destroy window, thread exits
run_project()
```

Communication via `std::sync::mpsc` channel:

```rust
enum SplashMessage {
    UpdateStatus(String),
    UpdateProgress(f32),       // 0.0 to 1.0
    Close,
}
```

### Graceful Degradation

- Window creation failure (headless, no display): bootstrap proceeds silently
- Splash thread panic: main thread catches and continues
- Splash is purely cosmetic; no bootstrap logic depends on it

## Feature Flag

```toml
[features]
default = []
splash = ["eframe", "egui", "image"]

[dependencies]
eframe = { version = "0.29", optional = true }
egui = { version = "0.29", optional = true }
image = { version = "0.25", optional = true, default-features = false, features = ["png", "jpeg"] }
```

When `splash` is disabled, all splash functions compile to no-ops.

## Configuration

All via environment variables read in `build.rs`, following PyApp's established pattern.

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PYAPP_SPLASH_ENABLED` | No | `false` | `true` or `1` to enable splash screen |
| `PYAPP_SPLASH_IMAGE` | No | None | Path to PNG/JPEG logo. Embedded via `include_bytes!`. Falls back to app name text. |
| `PYAPP_SPLASH_THEME` | No | `dark` | `dark` or `light`. Sets default colour scheme. |
| `PYAPP_SPLASH_BG_COLOR` | No | Theme default | Background colour (hex). Overrides theme. |
| `PYAPP_SPLASH_TEXT_COLOR` | No | Theme default | Text colour (hex). Overrides theme. |
| `PYAPP_SPLASH_PROGRESS_COLOR` | No | Theme default | Progress bar fill colour (hex). Overrides theme. |
| `PYAPP_SPLASH_WINDOW_TITLE` | No | `PYAPP_PROJECT_NAME` | Window title bar text |
| `PYAPP_SPLASH_WINDOW_WIDTH` | No | `480` | Window width in pixels |
| `PYAPP_SPLASH_WINDOW_HEIGHT` | No | `360` | Window height in pixels |

### Theme Defaults

| Property | Dark Theme | Light Theme |
|----------|-----------|-------------|
| Background | `#1a1a2e` | `#f5f5f5` |
| Text | `#ffffff` | `#1a1a1a` |
| Progress bar | `#4a90d9` | `#2a6cb6` |

Explicit colour env vars always override theme defaults.

## Bootstrap Phase Mapping

| Phase | Status Text | Weight |
|-------|-------------|--------|
| Distribution download | "Downloading Python {version}..." | 40% |
| Distribution extraction | "Extracting Python distribution..." | (part of above) |
| Embedded distribution extraction | "Extracting embedded distribution..." | (part of above) |
| Virtual environment creation | "Creating virtual environment..." | 10% |
| UV download | "Downloading UV package manager..." | 10% |
| pip download | "Downloading pip..." | 10% |
| Project installation (index) | "Installing {project_name}..." | 40% |
| Project installation (embedded) | "Installing embedded application..." | 40% |
| Project installation (deps file) | "Installing dependencies..." | 40% |

Within distribution download, actual byte-level progress is reported when the HTTP response includes a `Content-Length` header.

## Integration Points

### When splash appears

1. **First run** — installation directory does not exist
2. **`self restore`** — installation directory is removed then re-bootstrapped

### Code integration

1. **`distribution.rs::ensure_ready()`** — Start splash before `materialize()` if installation dir doesn't exist
2. **`distribution.rs::materialize()`** — Update status/progress at each phase
3. **`network.rs::download()`** — Report byte-level download progress to splash
4. **`distribution.rs::install_project()`** — Update status for installation phases
5. **`distribution.rs::run_project()`** — Close splash before executing application
6. **`commands/self_cmd/restore.rs`** — Splash also shown during restore

### build.rs additions

- Read `PYAPP_SPLASH_ENABLED`, conditionally enable `splash` feature
- Read `PYAPP_SPLASH_IMAGE`, validate file exists, embed path for `include_bytes!`
- Read `PYAPP_SPLASH_THEME` and all colour/dimension vars
- Pass as `cargo:rustc-env` directives
- Track image file via `cargo:rerun-if-changed`

## Window Layout

```
+------------------------------------------+
|            [Window Title]            [_X] |
+------------------------------------------+
|                                          |
|              +----------+                |
|              |   LOGO   |                |
|              |  IMAGE   |                |
|              +----------+                |
|                                          |
|           Application Name               |
|             v1.2.3                        |
|                                          |
|  [████████████░░░░░░░░░░░░░░░░░░]  45%  |
|                                          |
|        Downloading Python 3.13...        |
|                                          |
+------------------------------------------+
```

## Testing

### Unit tests
- Hex colour parsing (with/without `#` prefix)
- Theme default resolution and colour override logic
- Default configuration values
- Phase weight progress calculation
- Message channel send/receive

### Manual testing matrix
- First run with splash + logo image (Windows, macOS, Linux)
- First run with splash, no logo (text fallback)
- First run with splash disabled
- Subsequent run (no splash)
- `self restore` triggers splash
- Headless environment (no display)
- Ctrl+C during bootstrap
- Window close button during bootstrap
- Light theme / dark theme / custom colours

## Security

- No IPC or network exposure introduced
- Embedded image baked in at build time, not loaded from disk at runtime
- No user input accepted through splash window (display-only)
- Splash thread handles no sensitive data

## Scope Exclusions

- Animated images (GIF)
- Custom font embedding
- HTML/SVG template layouts
- pip/UV output parsing for per-package progress
- Localisation of status messages
