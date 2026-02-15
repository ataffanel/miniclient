# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Miniclient is a Rust GUI application for remotely controlling Crazyflie micro-drones via a gamepad. It connects to a drone over radio, streams its state estimates, and sends control setpoints from gamepad input.

## Commands

```bash
# Build
cargo build
cargo build --release

# Run
cargo run
cargo run --release

# Lint / Format
cargo clippy
cargo fmt

# Build distributable Linux AppImage (requires Podman/Docker)
./build-container.sh

# Build Windows MSI (requires cargo-wix and WiX Toolset v3)
cargo build --release
cargo wix --no-build --nocapture
# Output: target/wix/miniclient-<version>-x86_64.msi
```

There are no automated tests in this project.

## Architecture

The application has two source files that matter:

- [src/main.rs](src/main.rs) — all application logic (~426 lines)
- [ui/main.slint](ui/main.slint) — UI definition compiled at build time via [build.rs](build.rs)

### Concurrency Model

Three concurrent execution contexts run simultaneously:

1. **Slint event loop** (main thread) — handles UI events and a 50ms timer for syncing gamepad state to the UI
2. **Tokio async task** (`run_connection`) — manages the drone connection lifecycle: connects via radio URI, subscribes to a logging block for state estimates (roll/pitch/yaw), and sends commander setpoints at 50Hz
3. **Native thread** (`spawn_gamepad_thread`) — polls gamepads via `gilrs` at 100Hz; `gilrs` requires a non-Send context so it cannot run on the async executor

Cross-context state is shared via `Arc<Mutex<GamepadAxes>>` (gamepad axes) and `Arc<AtomicBool/AtomicUsize>` (enable flag, selected gamepad index).

### Control Flow

1. User enters a Crazyflie radio URI and clicks Connect
2. A `CancellationToken` is created; the connection task starts on the Tokio runtime
3. The connection task establishes a radio link, registers a logging block for drone state, then loops at 50Hz sending gamepad-derived setpoints
4. Gamepad sticks map to: left-X → roll (±30°), left-Y → pitch (±30°), right-X → yaw rate (±200°/s), right-Y → thrust (0–60000); deadzone is 0.1
5. Disconnect cancels the token, the task stops motors and exits
6. The UI timer (50ms) syncs the currently selected gamepad index from the UI to the shared atomic and refreshes the setpoint display

### Key Constants (src/main.rs)

| Constant | Value |
|---|---|
| `MAX_ROLL` | 30° |
| `MAX_PITCH` | 30° |
| `MAX_YAWRATE` | 200°/s |
| `MAX_THRUST` | 60000 |
| `DEADZONE` | 0.1 |

### AppImage Build (Linux)

[build-container.sh](build-container.sh) builds a Podman/Docker image from [Containerfile](Containerfile), compiles in release mode inside the container, and produces an architecture-aware AppImage (x86_64 or aarch64) using `linuxdeploy` with the Qt plugin. The Qt backend is required on Linux (`SLINT_BACKEND=qt`).

### Windows MSI

[wix/main.wxs](wix/main.wxs) is the WiX v3 installer definition used by `cargo-wix`. It installs the binary to `Program Files\Miniclient\` and creates a Start Menu shortcut. The EULA dialog is skipped (no RTF license file). The `UpgradeCode` GUID is fixed so upgrades replace the previous version cleanly.

Prerequisites for local MSI builds: install `cargo-wix` (`cargo install cargo-wix`) and WiX Toolset v3 (available via `winget install WiXToolset.WiXToolset`, requires admin).

Releases are automated via [.github/workflows/release.yml](.github/workflows/release.yml): pushing a tag matching `[0-9]*.[0-9]*.[0-9]*` (e.g. `0.1.0`) triggers a build on `windows-latest`, installs WiX v3, produces the MSI, and attaches it to a GitHub Release.
