#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Build with: cargo build -p nanoclick --features eframe/__screenshot --locked
# The dependency's capture hook exits after the second frame; no run is started.
output="${1:-/tmp/nanoclick-ui.png}"
scale="${3:-1}"
case "$scale" in 1|1.5|2) ;; *) echo "Scale must be 1, 1.5 or 2" >&2; exit 2 ;; esac
capture_config="$(mktemp -d /tmp/nanoclick-capture.XXXXXX)"
trap 'rmdir "$capture_config" 2>/dev/null || true' EXIT
xvfb-run -a -s '-screen 0 2048x1536x24 -nolisten tcp' env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 WINIT_X11_SCALE_FACTOR="$scale" XDG_CONFIG_HOME="$capture_config" EFRAME_SCREENSHOT_TO="$output" target/debug/nanoclick --theme "${2:-dark}"
