#!/usr/bin/env python3
"""
Generate keyboard controller TOML files with proper positioning.

This script generates controller TOML files for different keyboard layouts
(QWERTY, Colemak, etc.) that use the Base Generic shared images.

The positioning uses pad_center_x/y to place individual key images (59x53 pixels)
on the base keyboard image (1156x300 pixels).

Layout structure:
- Base image: 1156x300 pixels (from Base Generic/Base.webp)
- Individual key images: 59x53 pixels each (from Base Generic/Buttons/)
- Grid: approximately 19 keys wide x 5 keys tall
- pad_center_x/y: center position of each key on the base image

Usage:
    python3 generate_keyboard_layout.py

This will create:
- /home/melechtna/.config/scf-next/controllers/Keyboard & Mouse/Generic New/controller.toml
  - Uses Base Generic/Base.webp as the background
  - Uses Base Generic/Buttons/*.webp as individual key images
  - Has pad_center_x/y for each key positioned on the base image
  
The existing Generic profile is NOT modified. Update other profiles (Colemak, Gaming)
manually or by running the script with appropriate options.
"""

import os
import sys
import shutil

# Configuration - these paths are relative to the scf-next data directories
BASE_IMAGE = "Base Generic/Base.webp"
BUTTONS_DIR = "Base Generic/Buttons"
KEY_WIDTH = 59  # pixels
KEY_HEIGHT = 53  # pixels

# Calculate grid dimensions based on base image size
BASE_WIDTH = 1156
BASE_HEIGHT = 300
COLS = BASE_WIDTH // KEY_WIDTH  # 19 keys across
ROWS = BASE_HEIGHT // KEY_HEIGHT  # 5 keys tall

# QWERTY key positions: key_id -> (row, col) 0-indexed
QWERTY_KEY_POSITIONS = {
    # Row 0: Tilde / Number row (topmost)
    "tidle": (0, 0),
    "k_1": (0, 1),
    "k_2": (0, 2),
    "k_3": (0, 3),
    "k_4": (0, 4),
    "k_5": (0, 5),
    "k_6": (0, 6),
    "k_7": (0, 7),
    "k_8": (0, 8),
    "k_9": (0, 9),
    "k_0": (0, 10),
    "k_minus": (0, 11),
    "k_equals": (0, 12),

    # Row 1: Tab / QWERTY top row
    "k_tab": (1, 0),
    "k_q": (1, 1),
    "k_w": (1, 2),
    "k_e": (1, 3),
    "k_r": (1, 4),
    "k_t": (1, 5),
    "k_y": (1, 6),
    "k_u": (1, 7),
    "k_i": (1, 8),
    "k_o": (1, 9),
    "k_p": (1, 10),
    "k_openbracket": (1, 11),
    "k_closebracket": (1, 12),
    "k_backslash": (1, 13),

    # Row 2: Caps / ASDF
    "k_cpslck": (2, 0),
    "k_a": (2, 1),
    "k_s": (2, 2),
    "k_d": (2, 3),
    "k_f": (2, 4),
    "k_g": (2, 5),
    "k_h": (2, 6),
    "k_j": (2, 7),
    "k_k": (2, 8),
    "k_l": (2, 9),
    "k_semicolon": (2, 10),
    "k_apostrophe": (2, 11),
    "k_enter": (2, 12),

    # Row 3: Shift / ZXCV
    "k_lshift": (3, 0),
    "k_z": (3, 1),
    "k_x": (3, 2),
    "k_c": (3, 3),
    "k_v": (3, 4),
    "k_b": (3, 5),
    "k_n": (3, 6),
    "k_m": (3, 7),
    "k_comma": (3, 8),
    "k_period": (3, 9),
    "k_forslash": (3, 10),
    "k_rshift": (3, 11),

    # Row 4: Ctrl / Super / Alt / Space / AltGr / Super / Context / Ctrl
    "k_lctrl": (4, 0),
    "k_lsuper": (4, 1),
    "k_lalt": (4, 2),
    "k_spc": (4, 3),  # Space is wide - use center position
    "k_ralt": (4, 4),
    "k_rsuper": (4, 5),
    "k_context": (4, 6),
    "k_rctrl": (4, 7),
}


def key_position_to_pad_center(row, col):
    """Convert grid position to pad_center_x/y values.
    
    The pad_center is the center of the key on the base image.
    x = col * key_width + key_width / 2
    y = row * key_height + key_height / 2
    """
    x = col * KEY_WIDTH + KEY_WIDTH / 2
    y = row * KEY_HEIGHT + KEY_HEIGHT / 2
    return x, y


def float_to_str(f):
    """Convert float to TOML-compatible string with one decimal place."""
    return f"{f:.1f}"


def get_controller_dir(base_dir, profile_name):
    """Get the controller directory for a profile, ensuring it exists."""
    return os.path.join(base_dir, profile_name)


def generate_controller_toml(layout_name, key_positions,
                              output_dir=None,
                              controller_base_dir="/home/melechtna/.config/scf-next/controllers/Keyboard & Mouse"):
    """Generate a controller.toml for the given layout.
    
    Args:
        layout_name: Name of the layout (e.g., "Generic New")
        key_positions: Dict of key_id -> (row, col)
        output_dir: Specific output directory (overrides default)
        controller_base_dir: Base directory for controllers
    
    Returns:
        Path to the generated TOML file, or None if failed
    """
    
    # Determine output directory
    if output_dir:
        toml_path = os.path.join(output_dir, "controller.toml")
    else:
        toml_path = os.path.join(get_controller_dir(controller_base_dir, layout_name), "controller.toml")
    
    # Check if file already exists and ask (or just proceed with warning)
    if os.path.exists(toml_path):
        print(f"WARNING: {toml_path} already exists. Overwriting.")
    
    lines = []
    lines.append("[controller]")
    lines.append(f'name = "Keyboard & Mouse ({layout_name})"')
    lines.append(f'base = "{BASE_IMAGE}"')
    lines.append('icon = "images/Icon.webp"')
    lines.append(f'color = "#e5a50aFF"')
    lines.append("")
    lines.append("[defaults]")
    lines.append('paint = "standard"')
    lines.append("")
    lines.append("# ── Keyboard layout (US ANSI) ──")
    lines.append("# Row 1: tilde / number row")
    
    # Generate button entries for each key
    for key_id, (row, col) in sorted(key_positions.items()):
        pad_x, pad_y = key_position_to_pad_center(row, col)
        # Image path uses Base Generic individual key buttons
        image_path = f"{BUTTONS_DIR}/{key_id}.webp"
        
        pad_x_str = float_to_str(pad_x)
        pad_y_str = float_to_str(pad_y)
        
        btn_line = "[[buttons]]"
        btn_line += f'\nid = "{key_id}"'
        btn_line += f'\nimage = "{image_path}"'
        # Input type - keyboard with scancode; placeholder value
        # In practice, these would be mapped to actual SDL scancodes
        # for the specific layout (Colemak, QWERTY, etc.)
        btn_line += '\ninput = { type = "keyboard", key = 0 }'
        btn_line += f'\n travel = 15.0'
        btn_line += f'\n paint = "standard"'
        btn_line += f'\n transparency = 1.0'
        # Add pad_center for positioning on the base image
        btn_line += f'\n pad_center_x = {pad_x_str}'
        btn_line += f'\n pad_center_y = {pad_y_str}'
        
        lines.append(btn_line)
    
    lines.append("")
    lines.append("# ── Mouse buttons ──")
    
    # Standard mouse buttons using Base Generic images
    mouse_buttons = [
        ("m_left", f"{BUTTONS_DIR}/lclk.webp", "mouse", 0),
        ("m_right", f"{BUTTONS_DIR}/rclk.webp", "mouse", 1),
    ]
    
    for btn_id, image, input_type, button_num in mouse_buttons:
        lines.append("[[buttons]]")
        lines.append(f'id = "{btn_id}"')
        lines.append(f'image = "{image}"')
        if input_type == "mouse":
            lines.append(f'input = {{ type = "mouse", button = {button_num} }}')
        lines.append(' travel = 15.0')
        lines.append(' paint = "standard"')
        lines.append(' transparency = 1.0')
        lines.append("")
    
    # Scroll wheel buttons
    lines.append("[[buttons]]")
    lines.append('id = "m_middle"')
    lines.append(f'image_up = "{BUTTONS_DIR}/scrlwu.webp"')
    lines.append(f'image_down = "{BUTTONS_DIR}/scrlwd.webp"')
    lines.append('input = { type = "scroll_wheel" }')
    lines.append(' travel = 15.0')
    lines.append(' paint = "standard"')
    lines.append(' transparency = 1.0')
    lines.append("")
    
    lines.append("# ── Mouse movement (virtual stick driven by physical mouse) ──")
    lines.append("[[buttons]]")
    lines.append('id = "m_move"')
    lines.append('input = { type = "mouse_move" }')
    lines.append(' paint = "mouse"')
    lines.append(' pad_size_x = 118')
    lines.append(' pad_size_y = 118')
    lines.append(' pad_center_x = 1069')
    lines.append(' pad_center_y = 201')
    
    # Join all lines
    content = "\n".join(lines)
    
    # Ensure output directory exists
    toml_dir = os.path.dirname(toml_path)
    os.makedirs(toml_dir, exist_ok=True)
    
    # Write TOML file
    with open(toml_path, "w") as f:
        f.write(content + "\n")
    
    print(f"Written {toml_path}")
    return toml_path


def generate_qwerty_layout(controller_base_dir="/home/melechtna/.config/scf-next/controllers/Keyboard & Mouse"):
    """Generate the Generic New QWERTY layout profile."""
    
    layout_name = "Generic New"
    toml_path = generate_controller_toml(
        layout_name, 
        QWERTY_KEY_POSITIONS,
        controller_base_dir=controller_base_dir
    )
    
    print(f"\n=== {layout_name} (QWERTY) generated ===")
    print(f"File: {toml_path}")
    print("\nKey features:")
    print("- Base image: Base Generic/Base.webp (1156x300)")
    print("- Key images: Base Generic/Buttons/*.webp (59x53 each)")
    print("- pad_center_x/y positions each key on the base image")
    print("- input.key can be remapped for Colemak, etc.")
    print("- Mouse buttons use Base Generic images too")
    
    return toml_path


def generate_all_layouts(controller_base_dir="/home/melechtna/.config/scf-next/controllers/Keyboard & Mouse"):
    """Generate multiple layout profiles."""
    
    layouts = {
        "Generic New": QWERTY_KEY_POSITIONS,
        # Add more layouts here as needed:
        # "Colemak": COLEMAK_KEY_POSITIONS,
        # "Gaming": GAMING_KEY_POSITIONS,
    }
    
    generated = []
    for layout_name, key_positions in layouts.items():
        toml_path = generate_controller_toml(
            layout_name, 
            key_positions,
            controller_base_dir=controller_base_dir
        )
        generated.append((layout_name, toml_path))
    
    print(f"\nGenerated {len(generated)} layout profiles:")
    for name, path in generated:
        print(f"  - {name}: {path}")
    
    return generated


if __name__ == "__main__":
    print("Keyboard Layout Positioning Generator")
    print("=" * 50)
    print()
    print(f"Configuration:")
    print(f"  Base image: {BASE_IMAGE}")
    print(f"  Key images: {BUTTONS_DIR}/ (59x53 pixels each)")
    print(f"  Base size: 1156x300 pixels")
    print(f"  Grid: {COLS} keys wide x {ROWS} keys tall")
    print()
    print("This script will generate the 'Generic New' profile using")
    print("the Base Generic shared images with proper positioning.")
    print("The existing 'Generic' profile is NOT modified.")
    print()
    print("=" * 50)
    print()
    
    # Generate the Generic New QWERTY layout
    toml_path = generate_qwerty_layout()
    
    print()
    print("=" * 50)
    print("Generation complete!")
    print()
    print("Next steps:")
    print("1. The 'Generic New' profile is now available in the controller selector")
    print("2. Key mappings (input.key) can be remapped for Colemak, Dvorak, etc.")
    print("3. pad_center_x/y values can be adjusted if key images need repositioning")
    print("4. To create additional layouts, add key position maps and re-run")