#!/usr/bin/env bash
# OmaTUNES Quickshell Module Installer

set -e

DEST_DIR="$HOME/.config/quickshell/modules/omatunes"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Installing OmaTUNES Quickshell Player module..."
mkdir -p "$DEST_DIR"

cp "$SCRIPT_DIR/OmatunesWidget.qml" "$DEST_DIR/"
cp "$SCRIPT_DIR/OmatunesPopup.qml" "$DEST_DIR/"

# Also install Omarchy plugin files if Omarchy is present
OMARCHY_DIR="$HOME/.config/omarchy/plugins/omatunes"
if [ -d "$HOME/.config/omarchy" ] || [ -f "$SCRIPT_DIR/BarWidget.qml" ]; then
    mkdir -p "$OMARCHY_DIR"
    cp "$SCRIPT_DIR/BarWidget.qml" "$OMARCHY_DIR/" 2>/dev/null || true
    cp "$SCRIPT_DIR/manifest.json" "$OMARCHY_DIR/" 2>/dev/null || true
    echo "Omarchy v4 Plugin files copied to $OMARCHY_DIR"
fi

echo "Files copied to $DEST_DIR"
echo ""
echo "Installation complete!"
echo "Add the following lines to your ~/.config/quickshell/shell.qml:"
echo ""
echo '  import "./modules/omatunes"'
echo '  ...'
echo '  OmatunesWidget {}'
