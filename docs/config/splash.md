# Splash screen configuration

-----

The splash screen provides visual feedback during the first-run bootstrap process, showing a progress bar and status messages while the Python distribution is downloaded and your application's dependencies are installed. It appears automatically on first run and during `self restore`, and closes when bootstrap completes.

!!! note
    The splash screen is part of the [Cognisn fork](https://github.com/cognisn/pyapp) of PyApp. It is not available in upstream PyApp.

## Enabling

To enable the splash screen, you must do both:

1. Set the `PYAPP_SPLASH_ENABLED` environment variable to `true` or `1`
2. Build with the `splash` Cargo feature: `cargo build --release --features splash`

The environment variable configures the splash screen at build time (theme, colours, image), while the Cargo feature links the required GUI dependencies (`eframe`/`egui`/`image`, ~1.5MB binary size increase).

When the feature is not enabled, no GUI dependencies are included and the binary is identical to upstream PyApp.

## Theme

You may set the `PYAPP_SPLASH_THEME` option to `dark` (default) or `light` to control the colour scheme.

| Property | Dark theme | Light theme |
|----------|-----------|-------------|
| Background | `#1a1a2e` | `#f5f5f5` |
| Text | `#ffffff` | `#1a1a1a` |
| Progress bar | `#4a90d9` | `#2a6cb6` |

Theme defaults can be overridden individually using the colour options below.

## Colours

All colour values are specified as hex strings, with or without a `#` prefix.

### Background

You may set the `PYAPP_SPLASH_BG_COLOR` option to override the background colour. Defaults to the theme's background colour.

### Text

You may set the `PYAPP_SPLASH_TEXT_COLOR` option to override the text colour used for the application name, version, status messages, and progress percentage. Defaults to the theme's text colour.

### Progress bar

You may set the `PYAPP_SPLASH_PROGRESS_COLOR` option to override the progress bar fill colour. Defaults to the theme's progress colour.

## Logo image

You may set the `PYAPP_SPLASH_IMAGE` option to the path of a PNG or JPEG image file. The image is embedded into the binary at build time and displayed as the logo in the splash window.

If not set, the splash screen displays the application name (from `PYAPP_PROJECT_NAME`) as large text instead.

!!! tip
    The logo is scaled to fit within 60% of the window width and 40% of the window height, maintaining aspect ratio.

## Window

### Title

You may set the `PYAPP_SPLASH_WINDOW_TITLE` option to control the window title bar text. Defaults to the value of `PYAPP_PROJECT_NAME`.

### Dimensions

You may set the `PYAPP_SPLASH_WINDOW_WIDTH` and `PYAPP_SPLASH_WINDOW_HEIGHT` options to control the splash window size in pixels. The defaults are `480` and `360` respectively.

## Summary

| Option | Default | Description |
|--------|---------|-------------|
| `PYAPP_SPLASH_ENABLED` | `false` | Set to `true` or `1` to enable |
| `PYAPP_SPLASH_THEME` | `dark` | `dark` or `light` |
| `PYAPP_SPLASH_BG_COLOR` | Theme default | Background colour (hex) |
| `PYAPP_SPLASH_TEXT_COLOR` | Theme default | Text colour (hex) |
| `PYAPP_SPLASH_PROGRESS_COLOR` | Theme default | Progress bar colour (hex) |
| `PYAPP_SPLASH_IMAGE` | None | Path to PNG/JPEG logo file |
| `PYAPP_SPLASH_WINDOW_TITLE` | `PYAPP_PROJECT_NAME` | Window title |
| `PYAPP_SPLASH_WINDOW_WIDTH` | `480` | Window width in pixels |
| `PYAPP_SPLASH_WINDOW_HEIGHT` | `360` | Window height in pixels |

## Example

=== "Linux/macOS"

    ```bash
    export PYAPP_PROJECT_NAME="myapp"
    export PYAPP_PROJECT_VERSION="1.0.0"
    export PYAPP_EXEC_MODULE="myapp"
    export PYAPP_IS_GUI="true"

    # Enable splash screen with custom branding
    export PYAPP_SPLASH_ENABLED="true"
    export PYAPP_SPLASH_THEME="dark"
    export PYAPP_SPLASH_IMAGE="/path/to/logo.png"
    export PYAPP_SPLASH_BG_COLOR="#1a1a2e"
    export PYAPP_SPLASH_PROGRESS_COLOR="#4a90d9"

    cargo build --release --features splash
    ```

=== "Windows (PowerShell)"

    ```powershell
    $env:PYAPP_PROJECT_NAME = "myapp"
    $env:PYAPP_PROJECT_VERSION = "1.0.0"
    $env:PYAPP_EXEC_MODULE = "myapp"
    $env:PYAPP_IS_GUI = "true"

    # Enable splash screen with custom branding
    $env:PYAPP_SPLASH_ENABLED = "true"
    $env:PYAPP_SPLASH_THEME = "dark"
    $env:PYAPP_SPLASH_IMAGE = "C:\path\to\logo.png"
    $env:PYAPP_SPLASH_BG_COLOR = "#1a1a2e"
    $env:PYAPP_SPLASH_PROGRESS_COLOR = "#4a90d9"

    cargo build --release --features splash
    ```

## Behaviour

- The splash screen only appears during first-run bootstrap and `self restore`. Subsequent runs skip it entirely.
- If the splash window fails to create (e.g. on a headless server with no display), bootstrap proceeds silently.
- The splash window runs on a separate thread and never blocks the bootstrap process.
- The window close button has no effect on the bootstrap process; closing the splash window early does not cancel the installation.
- No user input is accepted through the splash window; it is display-only.

## Cross-compilation

When using [cross](https://github.com/cross-rs/cross), all `PYAPP_SPLASH_*` environment variables are passed through to the build container automatically.

The splash screen uses the OpenGL (`glow`) renderer. Platform-specific GUI and OpenGL dependencies may be required:

- **Linux:** `libx11-dev`, `libxcb-dev`, `libgl1-mesa-dev` (or equivalents)
- **macOS:** Xcode Command Line Tools (OpenGL is provided by the system frameworks)
- **Windows:** No additional dependencies (OpenGL is provided by the graphics driver)
