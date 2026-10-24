#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
temporary=$(mktemp -d)
trap 'rm -rf -- "$temporary"' EXIT
# No includes or service directories: owner-loss cleanup must not auto-activate
# installed portals, keyrings, accessibility buses or desktop helpers.
cat > "$temporary/bus.conf" <<CONFIG
<busconfig>
  <type>session</type>
  <listen>unix:path=$temporary/bus</listen>
  <auth>EXTERNAL</auth>
  <policy user="$(id -u)">
    <allow own="*"/>
    <allow send_destination="*"/>
    <allow receive_sender="*"/>
  </policy>
</busconfig>
CONFIG
# Private directory/socket and no compositor: never use the user's session bus.
dbus-run-session --config-file="$temporary/bus.conf" -- env -u DISPLAY -u WAYLAND_DISPLAY NANOCLICK_ISOLATED_DBUS=1 cargo run --locked -p nanoclick-platform --example portal_lifecycle --features portal-tests
