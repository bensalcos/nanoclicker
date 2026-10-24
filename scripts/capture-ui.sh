#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Build with: cargo build -p nanoclick --features eframe/__screenshot --locked
# The dependency's capture hook exits after the second frame; no run is started.
output="${1:-/tmp/nanoclick-ui.png}"
capture_config="$(mktemp -d /tmp/nanoclick-capture.XXXXXX)"
trap 'rmdir "$capture_config" 2>/dev/null || true' EXIT
xvfb-run -a -s '-screen 0 1024x768x24 -nolisten tcp' env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 XDG_CONFIG_HOME="$capture_config" EFRAME_SCREENSHOT_TO="$output" target/debug/nanoclick --theme "${2:-dark}"
