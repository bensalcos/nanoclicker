#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Always create a private display; never inject into the inherited desktop.
exec xvfb-run -a -s '-screen 0 1024x768x24 -nolisten tcp' env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 NANOCLICK_ISOLATED_X11=1 cargo run --release --locked -p nanoclick-platform --example x11_acceptance --features x11-tests -- "${1:-60}" "${2:-0}"
