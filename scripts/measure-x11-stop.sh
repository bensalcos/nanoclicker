#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Never inherit a user display for automated injection.
exec python3 scripts/with-isolated-x11.py --screen 1024x768x24 -- env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 NANOCLICK_ISOLATED_X11=1 cargo run --release --locked -p nanoclick-platform --example x11_acceptance --features x11-tests -- --stop-latency
