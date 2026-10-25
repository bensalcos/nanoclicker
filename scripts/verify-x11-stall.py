#!/usr/bin/env python3
"""Block only owned Xvfb/FIFO resources; never connect to the host display."""
import os
from pathlib import Path
import selectors
import signal
import subprocess
import sys
import time

from x11_fixture import OwnedX11

ROOT = Path(__file__).resolve().parents[1]


def line(child, timeout=8):
    with selectors.DefaultSelector() as selector:
        selector.register(child.stdout, selectors.EVENT_READ)
        if not selector.select(timeout):
            raise RuntimeError("Fixture marker timed out")
        value = child.stdout.readline().strip()
        if not value:
            raise RuntimeError("Fixture exited: " + child.stderr.read())
        print(value, flush=True)
        return value


def blocked_poll(child, prefix, authority=False):
    deadline = time.monotonic() + 2
    while time.monotonic() < deadline:
        for task in Path(f"/proc/{child.pid}/task").glob("*"):
            try:
                if (task / "comm").read_text().startswith(prefix):
                    channel = (task / "wchan").read_text().strip()
                    if "poll" in channel or (authority and "wait_for_partner" in channel):
                        return channel
            except (FileNotFoundError, ProcessLookupError):
                pass
        if child.poll() is not None:
            raise RuntimeError("Fixture exited before native poll")
        time.sleep(0.005)
    raise RuntimeError("Native blocked-wait observation unavailable; case is not accepted")


def pause_owned_server(server):
    server.send_signal(signal.SIGSTOP)
    deadline = time.monotonic() + 1
    while time.monotonic() < deadline:
        if server.poll() is not None:
            raise RuntimeError("Private server exited before suspension")
        stopped = os.waitid(os.P_PID, server.pid, os.WSTOPPED | os.WNOHANG)
        if stopped is not None:
            if stopped.si_code != os.CLD_STOPPED:
                raise RuntimeError("Unexpected private-server wait state")
            return
        time.sleep(0.005)
    raise RuntimeError("Private server did not confirm suspension")


def case(name):
    with OwnedX11("640x480x24") as fixture:
        directory = fixture.directory
        server = fixture.process
        child = None
        try:
            env = fixture.environment()
            env.pop("WAYLAND_DISPLAY", None)
            authority = directory / "client-Xauthority"
            if name.startswith("authority"):
                os.mkfifo(authority, mode=0o600)
            else:
                authority.write_bytes(fixture.authority.read_bytes())
                authority.chmod(0o600)
            env["XAUTHORITY"] = str(authority)
            env.update(XDG_CONFIG_HOME=str(directory), NANOCLICK_ISOLATED_X11="1", NANOCLICK_X11_STALL_DRIVER="1")
            child = subprocess.Popen([str(ROOT / "target/debug/examples/x11_stall"), name], env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            if line(child) != "READY":
                raise RuntimeError("Fixture not prepared")
            if not name.startswith("authority"):
                pause_owned_server(server)
            child.stdin.write("paused\n")
            child.stdin.flush()
            if line(child) != "ARMED":
                raise RuntimeError("Fixture not armed")
            prefix = "nanoclick-setup" if name.startswith("authority") else ("nanoclick-x11" if name == "shortcut-registration" else "nanoclick-work")
            print("observed_worker_wait," + blocked_poll(child, prefix, name.startswith("authority")), flush=True)
            if name == "disconnect":
                server.kill()
                server.wait(timeout=3)
            child.stdin.write("observe\n" if name in ("input-no-stop", "authority-timeout") else "stop\n")
            child.stdin.flush()
            result = line(child, 5)
            # Resume even on the red regression, allowing old blocking Drop to finish.
            if not result.startswith("PASS "):
                server.send_signal(signal.SIGCONT)
                if name.startswith("authority"):
                    # Release only our FIFO so the unfixed worker can exit.
                    descriptor = os.open(authority, os.O_WRONLY | os.O_NONBLOCK)
                    os.close(descriptor)
            # Success requires shutdown while the server/FIFO remains blocked.
            stdout, stderr = child.communicate(timeout=5)
            if child.returncode or not result.startswith("PASS "):
                raise RuntimeError(result + "\n" + stdout + stderr)
        finally:
            if server is not None and server.poll() is None:
                server.send_signal(signal.SIGCONT)
            if child is not None and child.poll() is None:
                child.kill()
                child.wait()


def main():
    subprocess.run(["cargo", "build", "--locked", "-p", "nanoclick-platform", "--example", "x11_stall", "--features", "x11-tests"], cwd=ROOT, check=True)
    for name in (sys.argv[1:] or ("input", "input-no-stop", "preparation", "disconnect", "shortcut-registration", "authority", "authority-timeout", "authority-shortcut")):
        case(name)


if __name__ == "__main__":
    main()
