#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Xvfb chooses a free server; never inherit the user's display for injection.
exec python3 scripts/with-isolated-x11.py --screen 1024x768x24 -- env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 NANOCLICK_ISOLATED_X11=1 cargo run --locked -p nanoclick-platform --example x11_acceptance --features x11-tests
