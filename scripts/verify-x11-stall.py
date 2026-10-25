#!/usr/bin/env python3
"""Block only owned Xvfb/FIFO resources; never connect to the host display."""
import os
from pathlib import Path
import selectors
import shutil
import stat
import signal
import subprocess
import sys
import tempfile
import time

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
            except FileNotFoundError:
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
    with tempfile.TemporaryDirectory(prefix="nanoclick-x11-stall-") as directory:
        read_fd, write_fd = os.pipe()
        server = child = None
        owned_paths = {}
        try:
            # An empty synthetic screen, no host clients or configuration. TCP is disabled.
            server = subprocess.Popen(["Xvfb", "-displayfd", str(write_fd), "-ac", "-nolisten", "tcp", "-screen", "0", "640x480x24"], pass_fds=(write_fd,), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            os.close(write_fd)
            write_fd = None
            with selectors.DefaultSelector() as selector:
                selector.register(read_fd, selectors.EVENT_READ)
                if not selector.select(5):
                    raise RuntimeError("Private Xvfb did not become ready")
            number = os.read(read_fd, 64).decode().strip()
            if not number.isdecimal() or server.poll() is not None:
                raise RuntimeError("Invalid private display")
            executable = shutil.which("Xvfb")
            if executable is None or Path(f"/proc/{server.pid}/exe").resolve() != Path(executable).resolve():
                raise RuntimeError("Owned server executable was not Xvfb")
            socket = Path(f"/tmp/.X11-unix/X{number}")
            socket_inodes = set()
            for descriptor in Path(f"/proc/{server.pid}/fd").iterdir():
                try:
                    target = os.readlink(descriptor)
                    if target.startswith("socket:["):
                        socket_inodes.add(target[8:-1])
                except FileNotFoundError:
                    pass
            owned_sockets = set()
            for entry in Path("/proc/net/unix").read_text().splitlines()[1:]:
                fields = entry.split(maxsplit=7)
                if len(fields) == 8 and fields[6] in socket_inodes:
                    owned_sockets.add(fields[7])
            required_sockets = {str(socket), "@" + str(socket)}
            if not required_sockets.issubset(owned_sockets):
                raise RuntimeError("Created server must own both X11 connection transports")
            if str(socket) in owned_sockets:
                info = socket.lstat()
                if info.st_uid != os.getuid() or not stat.S_ISSOCK(info.st_mode):
                    raise RuntimeError("Unexpected private socket ownership/type")
                owned_paths[socket] = (info.st_dev, info.st_ino, info.st_ctime_ns)
            # Auto-display Xvfb may use no lock file. Preserve any unrelated one.
            lock = Path(f"/tmp/.X{number}-lock")
            if lock.exists():
                info = lock.lstat()
                if info.st_uid == os.getuid() and stat.S_ISREG(info.st_mode):
                    if lock.read_text().strip() == str(server.pid):
                        owned_paths[lock] = (info.st_dev, info.st_ino, info.st_ctime_ns)
            env = os.environ.copy()
            env.pop("WAYLAND_DISPLAY", None)
            authority = Path(directory) / "Xauthority"
            if name.startswith("authority"):
                os.mkfifo(authority, mode=0o600)
            else:
                authority.touch(mode=0o600)
            env["XAUTHORITY"] = str(authority)
            env.update(DISPLAY=":" + number, XDG_CONFIG_HOME=directory, NANOCLICK_ISOLATED_X11="1", NANOCLICK_X11_STALL_DRIVER="1")
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
            if server is not None and server.poll() is None:
                server.terminate()
                try:
                    server.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    server.kill()
                    server.wait()
            # SIGKILL can leave X11 paths behind. Remove only the exact resources
            # captured from our own server; preserve replacements or unknown files.
            for path, identity in owned_paths.items():
                try:
                    info = path.lstat()
                    if (info.st_dev, info.st_ino, info.st_ctime_ns) == identity:
                        path.unlink()
                except FileNotFoundError:
                    pass
            os.close(read_fd)
            if write_fd is not None:
                os.close(write_fd)


def main():
    subprocess.run(["cargo", "build", "--locked", "-p", "nanoclick-platform", "--example", "x11_stall", "--features", "x11-tests"], cwd=ROOT, check=True)
    for name in (sys.argv[1:] or ("input", "input-no-stop", "preparation", "disconnect", "shortcut-registration", "authority", "authority-timeout", "authority-shortcut")):
        case(name)


if __name__ == "__main__":
    main()
