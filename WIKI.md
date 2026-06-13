# Controller Profile Markup Reference

> **Debug Key — `~` (backtick/tilde)**
>
> Press the tidle key at the main menu to reveal a **Controller Debug** card. Opening it shows a live viewer displaying every button, axis, hat, cap sense, and touchpad value from any connected SDL device. Use this to discover the correct button indices, axis numbers, hat directions, and VID/PID values when creating a new controller profile.

---

## Directory Structure

Profiles live under the `controllers/` directory. The folder hierarchy defines grouping in the menu:

```
controllers/
  VendorName/                 ← appears as a group card
    ControllerModel/          ← subgroup
      ColorVariant/           ← leaf: contains controller.toml
        controller.toml       ← profile definition
        images/
          Base.webp           ← full-canvas background image
          Icon.webp           ← menu thumbnail (200×200 recommended)
          Buttons/
            A.webp            ← per-button overlay images
            B.webp
            ...
```

The menu shows one level of nesting per screen. `VendorName` becomes the group card. Controllers under it are listed when the group is clicked.

---

## TOML Reference

### `[controller]` — Top-Level Metadata

```toml
[controller]
name = "Steam Controller v2"   # Display name in the Title Bar
width = 852                    # Canvas width in pixels (not recommended, derived from Base.webp)
height = 621                   # Canvas height in pixels (not recommended, derived from Base.webp)
base = "images/Base.webp"      # Background image (required)
icon = "images/Icon.webp"      # Menu card icon (required)
color = "#e5a50aFF"            # Default paint tint color with alpha (optional - gui fills this)
vendor = "28de"                # USB vendor ID in hex (required)
product = "1304"               # USB product ID in hex (required)
```

All button overlay images are **full-canvas transparent WebP** files. They are drawn on top of the base image at (0, 0) and  buttons are tinted with the paint color when pressed. Canvas resolution is determined by the base image dimensions. Lossless webp is recommended.

### `[defaults]` — Per-Profile Defaults

```toml
[defaults]
x = 0                          # Unused (reserved)
y = 0                          # Unused (reserved)
width = 852                    # Unused (reserved)
height = 621                   # Unused (reserved)
travel = 15                    # Default stick cap travel in pixels
paint = "standard"             # Default paint type for all buttons
transparency = 1.0             # Default opacity of paint (0.0 – 1.0)
```

### `[[buttons]]` — Button Definitions

Each `[[buttons]]` entry defines one overlay element. They are rendered as transparent overlays tinted with the paint color on press.

| Field | Type | Default | Description |
|---|---|---|---|
| `id` | string | **required** | Unique identifier (e.g. `"a"`, `"ls"`, `"dpad_up"`) |
| `image` | string | inherited | Path to the button overlay image |
| `image_up` | string | — | Scroll wheel top half image |
| `image_down` | string | — | Scroll wheel bottom half image |
| `input` | table | — | Input binding (see Input Types below) |
| `paint` | string | `"standard"` | Paint behavior (see Paint Types below) |
| `transparency` | float | `1.0` | Per-button opacity override |
| `travel` | float | `[defaults]` | Sticks level of travel |
| `capsense` | integer | — | Capacitive sense index (0–3): LeftGrip=0, RightGrip=1, LeftStick=2, RightStick=3 |
| `reference` | string | — | Inherit image from another `id` |
| `pad_size_x` | float | — | Touchpad tracking area width |
| `pad_size_y` | float | — | Touchpad tracking area height |
| `pad_angle` | float | — | Touchpad rotation in degrees |
| `pad_shape` | string | `"circle"` | `"circle"`, `"square"`, or `"squircle"` |
| `pad_center_x` | float | — | Touchpad center X on canvas |
| `pad_center_y` | float | — | Touchpad center Y on canvas |
| `pad_sensitivity` | float | — | Touchpad pressure sensitivity threshold |

---

## Input Types

The `input` field uses a tagged TOML inline table. The `type` tag selects the variant:

### `button` — Standard Digital Button

```toml
input = { type = "button", index = 0 }
```

`index` is the SDL gamepad button index. See [SDL Button Indices](#sdl-button-indices) below.

### `trigger` — Analog Trigger

```toml
input = { type = "trigger", axis = 4, threshold = 0.1 }
```

| Field | Default | Description |
|---|---|---|
| `axis` | **required** | SDL axis index (4 = left trigger, 5 = right trigger) |
| `threshold` | `0.1` | Normalized activation threshold (for `isPressed`); `getAnalogValue` returns the raw normalized value for progressive paint regardless of threshold |

### `stick` — Analog Stick

```toml
input = { type = "stick", axis_x = 0, axis_y = 1, press_button = 7 }
```

| Field | Default | Description |
|---|---|---|
| `axis_x` | **required** | SDL axis for horizontal deflection (0 = left X, 2 = right X) |
| `axis_y` | **required** | SDL axis for vertical deflection (1 = left Y, 3 = right Y) |
| `press_button` | — | Optional button index for stick click |

Stick caps are offset by `travel` pixels multiplied by the axis value, drawn in a second rendering pass after all non-stick buttons.

### `touchpad` — Touchpad

```toml
input = { type = "touchpad", index = 0, press_button = 20 }
```

| Field | Default | Description |
|---|---|---|
| `index` | **required** | SDL touchpad index |
| `press_button` | — | Optional physical click button index |

Used with `paint = "trackpad"` to draw a finger-position dot, or `paint = "progressive"` for pressure-based opacity.

### `keyboard` — Keyboard Key

```toml
input = { type = "keyboard", key = 16 }
```

`key` is an evdev scancode. Common values:

| Key | Code | Key | Code | Key | Code |
|---|---|---|---|---|---|
| `A` | 30 | `K` | 37 | `U` | 22 |
| `B` | 48 | `L` | 38 | `V` | 47 |
| `C` | 46 | `M` | 50 | `W` | 17 |
| `D` | 32 | `N` | 49 | `X` | 45 |
| `E` | 18 | `O` | 24 | `Y` | 21 |
| `F` | 33 | `P` | 25 | `Z` | 44 |
| `G` | 34 | `Q` | 16 | `Space` | 57 |
| `H` | 35 | `R` | 19 | `Shift` | 42 |
| `I` | 23 | `S` | 31 | `Enter` | 28 |
| `J` | 36 | `T` | 20 | `Escape` | 1 |

### `mouse` — Mouse Button

```toml
input = { type = "mouse", button = 0 }
```

`button`: `0` = left, `1` = middle, `2` = right.

### `mouse_move` — Mouse Movement (Virtual Stick)

```toml
input = { type = "mouse_move" }
```

No extra fields. Maps mouse X/Y deltas to a dot position within the canvas. Used with `paint = "mouse"`.

### `scroll_wheel` — Scroll Wheel

```toml
input = { type = "scroll_wheel" }
```

No extra fields. Requires `image_up` and `image_down` on the button definition. The top/bottom halves flash independently when scrolling up/down.

### `hat` — Hat Switch (D-Pad)

```toml
input = { type = "hat", index = 0, direction = 1 }
```

| Field | Default | Description |
|---|---|---|
| `index` | **required** | Hat index (usually 0) |
| `direction` | **required** | Direction bitmask (see Hat Directions below) |

Hat direction bitmask:

| Value | Direction |
|---|---|
| `0` | Centered |
| `1` | Up |
| `2` | Right |
| `3` | Right-Up (diagonal) |
| `4` | Down |
| `5` | Down-Right |
| `6` | Down-Left |
| `7` | - |
| `8` | Left |
| `9` | Left-Up |
| `12` | Left-Down |

These correspond to the SDL hat values. Use the debug tool to find the correct value for your controller.

### `multi` — Multi-Button Combo

```toml
input = { type = "multi", indices = [11, 14] }
```

Requires ALL listed button indices to be pressed simultaneously. Commonly used for D-pad diagonal entries. When a multi-button is active, the **individual directional button paints are suppressed** (they still draw their base images, but their paint overlay is hidden).

### `none` — No Input Binding

```toml
input = { type = "none" }
```

Useful for purely decorative elements or buttons driven solely by `capsense` (e.g., grip sensors that paint when released via `paint = "inverse"`).

---

## Paint Types

The `paint` field controls how a button's overlay image is tinted:

| Value | Behavior |
|---|---|
| `"standard"` | On/off: fully tinted when pressed (based on max transparency), invisible when released |
| `"progressive"` | Analog: opacity follows the input value (trigger axis, touchpad pressure, cap sense) up to max trasnparency |
| `"inverse"` | Inverted: tinted when **released**, clear when pressed (e.g. grip sensors) |
| `"tap"` | Quick paint (4 frames) then back to unpainted (e.g. capsense taps) |
| `"trackpad"` | Draws a dot at the finger position within `pad_*` bounds (does not paint the button image) |
| `"mouse"` | Draws a dot at the mouse delta position (does not paint the button image) |
| `"none"` | Never painted |

---

## Modes

Modes allow a single controller profile to support multiple input protocols (e.g., Switch, Xbox, PS4) with different button mappings.

```toml
[[modes]]
name = "switch"
vendor = "057e"
product = "2009"

[[modes]]
name = "xbox"
vendor = "045e"
product = "02e0"

[mode_overrides.switch.buttons.a]
image = "images/Buttons/B.webp"     # Different face button layout for Switch, good for controllers that change, or can be changed to alternate layouts

[mode_overrides.xbox.buttons.a]
index = 0                            # Remap button index per mode
```

Mode detection works by comparing the connected device's USB vendor/product ID against each mode's `vendor`/`product` fields. The matching mode's name is passed to the frontend, which applies any `[mode_overrides]`.

---

## Hardware ID Matching

When a profile defines vendor/product IDs (either at the `[controller]` level or inside any `[[modes]]`), the app performs **strict hardware matching**:

1. On profile load, the app closes any previously-connected device
2. It opens only the device whose VID/PID matches one of the declared pairs
3. If no matching device is found, the overlay shows a disconnected state — it never falls through to a different controller
4. On device disconnect, the same strict matching is used for reconnection

This ensures that loading an Xbox profile never reads from a Steam Controller, even when both are plugged in.

To find your controller's VID/PID:
- Use the **Debug tool** (`~` in menu) — it lists all detected devices with their vendor/product hex values
- Linux: `lsusb` or `udevadm info -a -n /dev/input/event*`
- Windows: Device Manager → Properties → Hardware IDs

---

## SDL Button Indices

Standard gamepad button mapping:

| Index | Constant | Typical Use |
|---|---|---|
| 0 | `SDL_GAMEPAD_BUTTON_SOUTH` | A (Xbox) / B (Nintendo) / Cross (PS) |
| 1 | `SDL_GAMEPAD_BUTTON_EAST` | B (Xbox) / A (Nintendo) / Circle (PS) |
| 2 | `SDL_GAMEPAD_BUTTON_WEST` | X (Xbox) / Y (Nintendo) / Square (PS) |
| 3 | `SDL_GAMEPAD_BUTTON_NORTH` | Y (Xbox) / X (Nintendo) / Triangle (PS) |
| 4 | `SDL_GAMEPAD_BUTTON_BACK` | Back / Select / Share / Minus |
| 5 | `SDL_GAMEPAD_BUTTON_GUIDE` | Guide / Home / PS |
| 6 | `SDL_GAMEPAD_BUTTON_START` | Start / Menu / Plus |
| 7 | `SDL_GAMEPAD_BUTTON_LEFT_STICK` | Left stick click |
| 8 | `SDL_GAMEPAD_BUTTON_RIGHT_STICK` | Right stick click |
| 9 | `SDL_GAMEPAD_BUTTON_LEFT_SHOULDER` | LB / L1 / L |
| 10 | `SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER` | RB / R1 / R |
| 11 | `SDL_GAMEPAD_BUTTON_DPAD_UP` | D-pad up |
| 12 | `SDL_GAMEPAD_BUTTON_DPAD_DOWN` | D-pad down |
| 13 | `SDL_GAMEPAD_BUTTON_DPAD_LEFT` | D-pad left |
| 14 | `SDL_GAMEPAD_BUTTON_DPAD_RIGHT` | D-pad right |
| 15 | `SDL_GAMEPAD_BUTTON_MISC1` | Capture / Star / QAM |
| 16 | `SDL_GAMEPAD_BUTTON_RIGHT_PADDLE1` | Right back paddle 1 |
| 17 | `SDL_GAMEPAD_BUTTON_LEFT_PADDLE1` | Left back paddle 1 |
| 18 | `SDL_GAMEPAD_BUTTON_RIGHT_PADDLE2` | Right back paddle 2 |
| 19 | `SDL_GAMEPAD_BUTTON_LEFT_PADDLE2` | Left back paddle 2 |
| 20 | `SDL_GAMEPAD_BUTTON_TOUCHPAD` | Touchpad click |
| 21–25 | `MISC2`–`MISC6` | Extra vendor-specific buttons |

### Axis Indices

| Index | Constant | Description |
|---|---|---|
| 0 | `SDL_GAMEPAD_AXIS_LEFTX` | Left stick horizontal |
| 1 | `SDL_GAMEPAD_AXIS_LEFTY` | Left stick vertical |
| 2 | `SDL_GAMEPAD_AXIS_RIGHTX` | Right stick horizontal |
| 3 | `SDL_GAMEPAD_AXIS_RIGHTY` | Right stick vertical |
| 4 | `SDL_GAMEPAD_AXIS_LEFT_TRIGGER` | Left analog trigger |
| 5 | `SDL_GAMEPAD_AXIS_RIGHT_TRIGGER` | Right analog trigger |

---

## Image Requirements

- **All images are full-canvas transparent overlays** — they are drawn at (0, 0) covering the entire canvas
- The base image determines canvas dimensions
- Supported formats: WebP (recommended, lossless also recommended), PNG
- Button images should only have non-transparent pixels where the button graphic is
- The paint system multiplies the paint color with non-transparent pixels to create the paint effect

---

## Reference System

The `reference` field lets one button inherit properties from another:

```toml
[[buttons]]
id = "lp"
image = "images/Buttons/LP.webp"
input = { type = "touchpad", index = 0, press_button = 20 }
paint = "progressive"

[[buttons]]
id = "lt"
reference = "lp"
input = { type = "touchpad", index = 0 }
paint = "trackpad"
pad_size_x = 187
pad_center_x = 260
pad_center_y = 338
```

The `lt` button inherits `image` from `lp` and only overrides `input`, `paint`, and touchpad tracking fields. In the renderer, referenced buttons act as **modifiers** — the highest alpha from any modifier is used for the target button's paint.

---

## Example: Minimal Profile

```toml
[controller]
name = "My Controller"
base = "images/Base.webp"
icon = "images/Icon.webp"

[[modes]]
name = "default"
vendor = "1234"
product = "abcd"

[[buttons]]
id = "a"
image = "images/Buttons/A.webp"
input = { type = "button", index = 0 }

[[buttons]]
id = "b"
image = "images/Buttons/B.webp"
input = { type = "button", index = 1 }

[[buttons]]
id = "ls"
image = "images/Buttons/LS.webp"
input = { type = "stick", axis_x = 0, axis_y = 1, press_button = 7 }

[[buttons]]
id = "lt"
image = "images/Buttons/LT.webp"
input = { type = "trigger", axis = 4 }
paint = "progressive"
```

---

## Creating a New Profile (Quick Start)

1. **Find your controller's VID/PID** — press `~` in the app menu to open the debug tool, or use `lsusb`
2. **Create the directory structure** — `controllers/VendorName/ModelName/Variant/` Note: folder structure is not strict, it purely stops when it finds a correct toml configuration
3. **Create a base image** — a full-canvas WebP showing the controller silhouette/base that buttons are overlayed onto
4. **Create button overlays** — individual WebP files for each button at the same canvas resolution
5. **Write `controller.toml`** — start from a similar existing profile, add your VID/PID in `[[modes]]`
6. **Set `paint = "progressive"` on triggers** — analog inputs need this to show partial press, though this is optional
7. **Add D-pad diagonals** — use `type = "multi"` with the two adjacent direction indices, also optional if your controller lacks inbetweens
8. **Test in the app** — load your profile, verify all buttons light up correctly with the debug tool
