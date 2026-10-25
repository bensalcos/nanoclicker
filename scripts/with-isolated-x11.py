#!/usr/bin/env python3
"""Run a command only after its newly created Xvfb passes ownership checks."""
import argparse
import signal
import subprocess
from x11_fixture import OwnedX11


def interrupted(signum, _):
    raise KeyboardInterrupt(signum)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--screen", default="2048x1536x24",
                        choices=("640x480x24", "1024x768x24", "2048x1536x24"))
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        parser.error("Pass a command after --")
    signal.signal(signal.SIGTERM, interrupted)
    with OwnedX11(args.screen) as fixture:
        print(f"owned_x11_display,{fixture.display}; server_pid,{fixture.process.pid}", flush=True)
        return subprocess.call(command, env=fixture.environment())


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
