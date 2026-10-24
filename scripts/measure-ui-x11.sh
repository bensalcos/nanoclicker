#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
app_binary="${1:-target/release/nanoclick}"
shift || true
bench_config="$(mktemp -d /tmp/nanoclick-bench.XXXXXX)"
mkdir -p "$bench_config/nanoclick"
cat > "$bench_config/nanoclick/preferences.json" <<'JSON'
{"version":1,"interval_ms":10,"button":"left","count":null,"theme":"dark","shortcut":"F6"}
JSON
# Kept in /tmp for provenance; no user configuration is read or replaced.
xvfb-run -a -s '-screen 0 1024x768x24 -nolisten tcp' env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 XDG_CONFIG_HOME="$bench_config" NANOCLICK_ISOLATED_X11=1 cargo run --locked -p nanoclick-platform --example ui_performance --features x11-tests -- "$app_binary" "$@"
