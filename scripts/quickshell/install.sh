#!/usr/bin/env bash
# OmaTUNES Quickshell Module Installer

set -e

DEST_DIR="$HOME/.config/quickshell/modules/omatunes"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Installing OmaTUNES Quickshell Player module..."
mkdir -p "$DEST_DIR"

cp "$SCRIPT_DIR/OmatunesWidget.qml" "$DEST_DIR/"
cp "$SCRIPT_DIR/OmatunesPopup.qml" "$DEST_DIR/"

echo "Files copied to $DEST_DIR"
echo ""
echo "Installation complete!"
echo "Add the following lines to your ~/.config/quickshell/shell.qml:"
echo ""
echo '  import "./modules/omatunes"'
echo '  ...'
echo '  OmatunesWidget {}'
