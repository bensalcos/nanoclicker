#!/usr/bin/env bash
# Optional Fedora/Weston 15 GPU benchmark; no installation or host-seat access.
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo"

if [[ "${1:-}" == --inside ]]; then
  shift
  : "${NANOCLICK_GPU_SESSION:?Private GPU session required}"
  session="$NANOCLICK_GPU_SESSION"
  [[ "${XDG_RUNTIME_DIR:-}" == "$session/runtime" ]]
  [[ -S "$session/runtime/nanoclick-headless" ]]
  xserver=''
  child_cleanup() {
    result=$?
    if [[ -n "$xserver" ]]; then
      kill -TERM "$xserver" 2>/dev/null || true
      wait "$xserver" 2>/dev/null || true
    fi
    printf '%s\n' "$result" >"$session/status"
  }
  trap child_cleanup EXIT
  binary="$1"
  profile="$2"
  display_file="$session/display-number"
  env -u WAYLAND_SOCKET WAYLAND_DISPLAY=nanoclick-headless Xwayland \
    -displayfd 3 -fullscreen -geometry 2048x1536 -nolisten tcp -noreset \
    3>"$display_file" >"$session/xwayland.log" 2>&1 &
  xserver=$!
  for _ in {1..100}; do
    if [[ -s "$display_file" ]]; then break; fi
    kill -0 "$xserver"
    sleep 0.05
  done
  number=$(head -1 "$display_file")
  [[ "$number" =~ ^[0-9]+$ ]]
  [[ "$(readlink "/proc/$xserver/exe")" == "$(realpath "$(command -v Xwayland)")" ]]
  export DISPLAY=":$number"
  unset WAYLAND_DISPLAY WAYLAND_SOCKET LD_LIBRARY_PATH WESTON_MODULE_MAP
  export XDG_SESSION_TYPE=x11 WINIT_X11_SCALE_FACTOR=1 NANOCLICK_ISOLATED_X11=1
  export XDG_CONFIG_HOME="$session/config"
  glxinfo -B >"$session/glxinfo.txt"
  grep -q 'Accelerated: yes' "$session/glxinfo.txt"
  export NANOCLICK_FAILURE_CAPTURE="$session/failure.png"
  # Validate the complete UI flow before collecting any performance figures.
  set +e
  target/debug/examples/ui_x11_acceptance "$binary" >"$session/ui-validation.log" 2>&1
  result=$?
  if [[ "$result" == 0 && "$profile" != validate ]]; then
    mkdir -p "$XDG_CONFIG_HOME/nanoclick"
    printf '%s\n' '{"version":2,"interval_ms":10,"button":"left","count":10,"repeat_mode":"until_stopped","theme":"dark","shortcut":"F6"}' >"$XDG_CONFIG_HOME/nanoclick/preferences.json"
    arguments=("$binary")
    if [[ "$profile" == quick ]]; then arguments+=(--quick-profile); fi
    target/release/examples/ui_performance "${arguments[@]}" >"$session/measurement.log" 2>&1
    result=$?
  fi
  exit "$result"
fi

if [[ "$#" -lt 2 || "$#" -gt 3 || ( "$#" == 3 && "$3" != --quick-profile && "$3" != --validate-only ) ]]; then
  echo "Usage: $0 WESTON_PREFIX APP_BINARY [--quick-profile|--validate-only]" >&2
  exit 2
fi
prefix=$(realpath "$1")
binary=$(realpath "$2")
profile=full
if [[ "${3:-}" == --quick-profile ]]; then profile=quick; fi
if [[ "${3:-}" == --validate-only ]]; then profile=validate; fi
if [[ "$prefix" == *[[:space:]]* ]]; then
  echo 'Weston shell-client configuration requires a prefix without whitespace' >&2
  exit 2
fi
library_path="$prefix/lib64:$prefix/lib64/weston"
version=$(LD_LIBRARY_PATH="$library_path" "$prefix/bin/weston" --version)
[[ "$version" == 'weston 15.'* ]]
PYTHONPATH="$repo/scripts" python - "$binary" <<'PY'
import sys
from pathlib import Path
from package import verify_binary
verify_binary(Path(sys.argv[1]), "linux")
PY
cargo build --locked -p nanoclick-platform --example ui_x11_acceptance --features x11-tests
cargo build --release --locked -p nanoclick-platform --example ui_performance --features x11-tests
session=$(mktemp -d /tmp/nanoclick-gpu.XXXXXX)
mkdir "$session/runtime" "$session/config"
chmod 700 "$session/runtime"
printf '%s\n' "$version" >"$session/weston-version.txt"
cat >"$session/weston.ini" <<CONFIG
[core]
xwayland=false
[shell]
client=$prefix/libexec/weston-desktop-shell
locking=false
panel-position=none
background-type=tile
background-color=0xff202020
[input-method]
path=$prefix/libexec/weston-keyboard
CONFIG
cat >"$session/bus.conf" <<CONFIG
<busconfig>
  <type>session</type>
  <listen>unix:path=$session/runtime/bus</listen>
  <auth>EXTERNAL</auth>
  <policy user="$(id -u)">
    <allow own="*"/>
    <allow send_destination="*"/>
    <allow receive_sender="*"/>
  </policy>
</busconfig>
CONFIG
module_map="headless-backend.so=$prefix/lib64/libweston-15/headless-backend.so;gl-renderer.so=$prefix/lib64/libweston-15/gl-renderer.so;desktop-shell.so=$prefix/lib64/weston/desktop-shell.so"
guard=''
cleanup() {
  if [[ -n "$guard" ]]; then
    kill -TERM "$guard" 2>/dev/null || true
    wait "$guard" 2>/dev/null || true
  fi
  rm -rf -- "$session/runtime"
}
trap cleanup EXIT
echo "GPU evidence directory: $session"
timeout --signal=TERM --kill-after=5s 360s dbus-run-session --config-file="$session/bus.conf" -- \
  env -u DISPLAY -u WAYLAND_DISPLAY -u WAYLAND_SOCKET XDG_RUNTIME_DIR="$session/runtime" \
  NANOCLICK_GPU_SESSION="$session" LD_LIBRARY_PATH="$library_path" WESTON_MODULE_MAP="$module_map" \
  "$prefix/bin/weston" --backend=headless --renderer=gl --fake-seat --width=2048 --height=1536 \
  --socket=nanoclick-headless --idle-time=0 --config="$session/weston.ini" --log="$session/weston.log" \
  -- /bin/bash "$repo/scripts/measure-ui-gpu.sh" --inside "$binary" "$profile" \
  >"$session/compositor-console.log" 2>&1 &
guard=$!
while kill -0 "$guard" 2>/dev/null; do
  if [[ -f "$session/status" ]]; then
    kill -TERM "$guard" 2>/dev/null || true
    break
  fi
  sleep 0.05
done
wait "$guard" || true
guard=''
if [[ ! -f "$session/status" ]]; then
  echo "GPU setup failed or timed out; inspect $session" >&2
  exit 1
fi
if [[ -f "$session/ui-validation.log" ]]; then cat "$session/ui-validation.log"; fi
if [[ -f "$session/measurement.log" ]]; then cat "$session/measurement.log"; fi
exit "$(cat "$session/status")"
