#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
app_binary="$(realpath "${1:-target/release/nanoclick}")"
cargo build --locked -p nanoclick-platform --example ui_x11_acceptance --features x11-tests
probe_binary="$(realpath target/debug/examples/ui_x11_acceptance)"
docker build -f infra/qa/Dockerfile -t nanoclick-qa:ubuntu24.04 infra/qa
docker run --rm --network none --read-only --cap-drop ALL --security-opt no-new-privileges \
    --tmpfs /tmp:rw,nosuid,size=256m \
    -v "$app_binary:/app/nanoclick:ro" -v "$probe_binary:/qa/probe:ro" \
    nanoclick-qa:ubuntu24.04 /bin/sh -c '
        mkdir -p /tmp/runtime /tmp/config /tmp/cache
        chmod 700 /tmp/runtime
        export XDG_RUNTIME_DIR=/tmp/runtime XDG_CONFIG_HOME=/tmp/config XDG_CACHE_HOME=/tmp/cache
        xvfb-run -a -s "-screen 0 2048x1536x24 -nolisten tcp" env XDG_SESSION_TYPE=x11 NANOCLICK_ISOLATED_X11=1 /qa/probe /app/nanoclick
    '
