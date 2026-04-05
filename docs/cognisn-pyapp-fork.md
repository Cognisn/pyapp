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
