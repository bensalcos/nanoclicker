#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [ "$#" -gt 0 ]; then
    app_binary="$1"
else
    cargo build --locked -p nanoclick
    app_binary="target/debug/nanoclick"
fi
scale="${2:-1}"
case "$scale" in 1|1.5|2) ;; *) echo "Scale must be 1, 1.5 or 2" >&2; exit 2 ;; esac
ui_config="$(mktemp -d /tmp/nanoclick-ui-test.XXXXXX)"
trap 'rmdir "$ui_config" 2>/dev/null || true' EXIT
xvfb-run -a -s '-screen 0 2048x1536x24 -nolisten tcp' env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 WINIT_X11_SCALE_FACTOR="$scale" XDG_CONFIG_HOME="$ui_config" NANOCLICK_ISOLATED_X11=1 cargo run --locked -p nanoclick-platform --example ui_x11_acceptance --features x11-tests -- "$app_binary"
