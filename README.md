# SCF Next

A cross-platform controller overlay that renders a virtual controller on screen with animated button presses, analog triggers, touchpad tracking, capsense, and more. Supports gamepad input plus keyboard and mouse input.

Built with [Tauri](https://tauri.app) (Rust backend + HTML/JS/CSS frontend) and [SDL3](https://libsdl.org).

## Features

- **On-screen Controller Overlay** – transparent overlay making easy capture in OBS
- **Controller Creation** – create your own controller representation with farily simple configs
- **Wide Controller Support** – Using SDL3, the framework supports a wide variety of inputs, including the Steam Controller v2, included is Keyboard and Mouse using evdev and rdev
- **Configurable Representation** – Numerous paint types and tracking methods for things like trackpads, controller sticks, tap sensors, and more
- **Device Specification** – Configs are bound to specific controllers using hardware VID/PID
- **Debug tool** – live input viewer for any connected device (press `~` in the main menu) to aid in device addition/creation

> Screenshots coming soon.

## Building

### Prerequisites

- Rust toolchain (2021 edition)
- CMake + Ninja
- Linux: `libevdev-dev`
- Windows (cross-compile): MinGW (`x86_64-w64-mingw32-gcc`)

### Setup

```bash
git clone --recursive https://github.com/your-username/scf-next
cd scf-next

# If you cloned without --recursive:
git submodule update --init --recursive

# Build (Linux native)
cargo build --release

# Build (Windows cross-compile)
cargo build --release --target x86_64-pc-windows-gnu
```

The build compiles SDL3 statically from the `libs/SDL` submodule. SDL3 capsense support requires a git build (not yet in a release) — the submodule tracks SDL main.

## Install/Uninstall

For Linux, the repo includes a just file. All that is required to install the program (with its icons), is to simply run

```bash
just install
```

and to uninstall simply

```bash
just uninstall
```

## Usage

1. Launch the app – The selection menu will be provided
2. Browse controller groups by vendor, select your desired controller
3. The overlay renders the controller with transparency and input reacts as you interact with them

## Creating Controller Profiles

See the wiki for the full TOML markup reference. Profiles live in `controllers/Vendor/Model/Variant/controller.toml` with images in an `images/` subdirectory.

## Contribution

While I can't predict how the controller landscape will progress, I do not pretend that the methods used are likely perfect, and somewhere down the line, sub-moduling SDL3 to include the Steam Controller v2's capabilities will stop being a requirement. So, any contributions, ideas, adjustments, etc. Would be welcome.

## Note to MacOS users

While a binary will be provided, I have no idea how well it will work, if at all. I am merely working off assumptions.S
