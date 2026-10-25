#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [ "$#" -gt 0 ]; then
    capture_binary="$1"
else
    cargo build --locked -p nanoclick --example input_capture
    capture_binary=target/debug/examples/input_capture
fi
capture_directory="$(mktemp -d /tmp/nanoclick-capture-test.XXXXXX)"
trap 'rm -rf -- "$capture_directory"' EXIT
python3 scripts/with-isolated-x11.py --screen 2048x1536x24 -- env -u WAYLAND_DISPLAY \
    XDG_SESSION_TYPE=x11 XDG_CONFIG_HOME="$capture_directory" NANOCLICK_ISOLATED_X11=1 \
    cargo run --locked -p nanoclick-platform --example capture_window --features x11-tests -- \
    "$capture_binary" "$capture_directory/capture.jsonl"
