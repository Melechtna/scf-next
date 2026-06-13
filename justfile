# Show available tasks
default:
    @just --list

# Build the application in release mode
build:
    cargo build --release

# Install system-wide
install: build
    #!/usr/bin/env bash
    set -e

    echo "Installing binary to /usr/bin/scf-next..."
    sudo install -Dm755 target/release/scf-next /usr/bin/scf-next

    echo "Installing icon to /usr/share/icons/hicolor/scalable/apps/com.scf-next.app.svg..."
    sudo install -Dm644 icons/icon.svg \
        /usr/share/icons/hicolor/scalable/apps/com.scf-next.app.svg

    echo "Installing .desktop file to /usr/share/applications/com.scf-next.app.desktop..."
    sudo install -Dm644 desktop/com.scf-next.app.desktop \
        /usr/share/applications/com.scf-next.app.desktop

    echo "Installing metainfo file to /usr/share/metainfo/com.scf-next.app.metainfo.xml..."
    sudo install -Dm644 res/com.scf-next.app.metainfo.xml \
        /usr/share/metainfo/com.scf-next.app.metainfo.xml

    echo "Updating icon cache..."
    sudo gtk-update-icon-cache -f /usr/share/icons/hicolor

    echo "Installation complete."

# Uninstall system-wide
uninstall:
    #!/usr/bin/env bash
    set -e

    echo "Removing /usr/bin/scf-next (if it exists)..."
    sudo rm -f /usr/bin/scf-next

    echo "Removing /usr/share/applications/com.scf-next.app.desktop (if it exists)..."
    sudo rm -f /usr/share/applications/com.scf-next.app.desktop

    echo "Removing /usr/share/metainfo/com.scf-next.app.metainfo.xml (if it exists)..."
    sudo rm -f /usr/share/metainfo/com.scf-next.app.metainfo.xml

    echo "Removing icon /usr/share/icons/hicolor/scalable/apps/com.scf-next.app.svg (if it exists)..."
    sudo rm -f /usr/share/icons/hicolor/scalable/apps/com.scf-next.app.svg

    echo "Updating icon cache..."
    sudo gtk-update-icon-cache -f /usr/share/icons/hicolor

    echo "Uninstallation complete."
