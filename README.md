# Miniclient

[![Release](https://github.com/ataffanel/miniclient/actions/workflows/release.yml/badge.svg)](https://github.com/ataffanel/miniclient/actions/workflows/release.yml)

Minimal Crazyflie client for gamepad-based flight control. This is a proof of concept for building a Crazyflie client in Rust using [crazyflie-lib](https://crates.io/crates/crazyflie-lib) and [crazyflie-link](https://crates.io/crates/crazyflie-link), with cross-platform releases (Linux, macOS, Windows).

The UI is built with [Slint](https://slint.dev/). Gamepad input is handled by [gilrs](https://crates.io/crates/gilrs).

## Features

- Connect to a Crazyflie via radio URI
- Fly using a gamepad (roll, pitch, yaw, thrust)
- Display drone state estimates alongside setpoints

## Building

```bash
cargo build
cargo build --release
```

## Running

```bash
cargo run --release
```

Connect a Crazyradio, enter the radio URI (e.g. `radio://0/80/2M/E7E7E7E7E7`), select a gamepad, and click Connect.

## Gamepad mapping

| Stick   | Axis          | Range       |
|---------|---------------|-------------|
| Left X  | Roll          | +/-30 deg   |
| Left Y  | Pitch         | +/-30 deg   |
| Right X | Yaw rate      | +/-200 deg/s|
| Right Y | Thrust        | 0-60000     |

Deadzone: 0.1

## Releases

Pre-built packages are available on the [GitHub Releases](https://github.com/ataffanel/miniclient/releases) page:

- **Linux**: AppImage (x86_64, aarch64)
- **macOS**: DMG (x86_64, arm64)
- **Windows**: MSI installer

## License

MIT OR Apache-2.0
