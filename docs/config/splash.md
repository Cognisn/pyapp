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
