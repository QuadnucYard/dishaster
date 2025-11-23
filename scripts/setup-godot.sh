#!/bin/bash

GODOT_VERSION=4.5.1

set -euo pipefail
echo "Setting up Godot ${GODOT_VERSION}"
tmpdir="$(mktemp -d)"
GODOT_BASE_URL="https://github.com/godotengine/godot-builds/releases/download/${GODOT_VERSION}-stable"
RUNTIME_ZIP="${GODOT_BASE_URL}/Godot_v${GODOT_VERSION}-stable_linux.x86_64.zip"
TEMPLATES_TPZ="${GODOT_BASE_URL}/Godot_v${GODOT_VERSION}-stable_export_templates.tpz"

echo "Downloading Godot runtime: $RUNTIME_ZIP"
curl -L -o "$tmpdir/godot_runtime.zip" "$RUNTIME_ZIP"
unzip -q -o "$tmpdir/godot_runtime.zip" -d "$tmpdir"
# Find and install the runtime binary
godot_bin="$(find "$tmpdir" -maxdepth 1 -type f -executable -name 'Godot*' -print -quit || true)"
if [ -z "$godot_bin" ]; then
# fallback: try any executable
godot_bin="$(find "$tmpdir" -maxdepth 1 -type f -executable -print -quit || true)"
fi
if [ -z "$godot_bin" ]; then
echo "Error: Godot runtime binary not found in $tmpdir"
ls -la "$tmpdir"
exit 1
fi
echo "Installing Godot binary: $godot_bin"
sudo mv "$godot_bin" /usr/local/bin/godot
sudo chmod +x /usr/local/bin/godot
# Provide common alias for scripts/tools that expect 'godot4'
sudo ln -sf /usr/local/bin/godot /usr/local/bin/godot4 || true
# Verify installation
godot --version
echo "Godot setup complete"

echo "Downloading Godot export templates: $TEMPLATES_TPZ"
curl -L -o "$tmpdir/godot_export_templates.tpz" "$TEMPLATES_TPZ"
templates_dir="$HOME/.local/share/godot/export_templates"
mkdir -v -p "$templates_dir"
unzip -q -o "$tmpdir/godot_export_templates.tpz" -d "$templates_dir"
mv -v "$templates_dir/templates" "$templates_dir/${GODOT_VERSION}.stable"
# Verify templates installation
ls -la "$templates_dir/${GODOT_VERSION}.stable/"
