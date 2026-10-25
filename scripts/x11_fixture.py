"""Own and verify a private authenticated Xvfb; never select an existing display."""
import os
from pathlib import Path
import selectors
import shutil
import signal
import stat
import struct
import subprocess
import tempfile


class OwnedX11:
    def __init__(self, screen="2048x1536x24"):
        if screen not in ("640x480x24", "1024x768x24", "2048x1536x24"):
            raise ValueError("Unsupported fixture screen")
        self.screen = screen
        self.process = None
        self.paths = {}
        self.temporary = None

    def __enter__(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="nanoclick-owned-x11-")
        self.directory = Path(self.temporary.name)
        self.authority = self.directory / "Xauthority"
        # FamilyWild + empty display matches local filesystem/abstract sockets.
        fields = (b"", b"", b"MIT-MAGIC-COOKIE-1", os.urandom(32))
        with self.authority.open("xb") as authority:
            os.chmod(self.authority, 0o600)
            authority.write(struct.pack(">H", 65535))
            for field in fields:
                authority.write(struct.pack(">H", len(field)) + field)
        read_fd, write_fd = os.pipe()
        try:
            self.process = subprocess.Popen(
                ["Xvfb", "-displayfd", str(write_fd), "-auth", str(self.authority),
                 "-nolisten", "tcp", "-screen", "0", self.screen],
                pass_fds=(write_fd,), stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            os.close(write_fd)
            write_fd = None
            with selectors.DefaultSelector() as selector:
                selector.register(read_fd, selectors.EVENT_READ)
                if not selector.select(5):
                    raise RuntimeError("Owned Xvfb did not become ready")
            number = os.read(read_fd, 64).decode().strip()
            if not number.isdecimal() or self.process.poll() is not None:
                raise RuntimeError("Invalid owned display")
            executable = shutil.which("Xvfb")
            if executable is None or Path(f"/proc/{self.process.pid}/exe").resolve() != Path(executable).resolve():
                raise RuntimeError("Owned server executable was not Xvfb")
            self.display = ":" + number
            socket = Path(f"/tmp/.X11-unix/X{number}")
            inodes = set()
            for descriptor in Path(f"/proc/{self.process.pid}/fd").iterdir():
                try:
                    target = os.readlink(descriptor)
                    if target.startswith("socket:["):
                        inodes.add(target[8:-1])
                except FileNotFoundError:
                    pass
            owned_sockets = set()
            for entry in Path("/proc/net/unix").read_text().splitlines()[1:]:
                fields = entry.split(maxsplit=7)
                if len(fields) == 8 and fields[6] in inodes:
                    owned_sockets.add(fields[7])
            if not {str(socket), "@" + str(socket)}.issubset(owned_sockets):
                raise RuntimeError("Owned Xvfb must own both X11 connection transports")
            self._remember(socket, stat.S_ISSOCK)
            lock = Path(f"/tmp/.X{number}-lock")
            if lock.exists():
                info = lock.lstat()
                if info.st_uid == os.getuid() and stat.S_ISREG(info.st_mode):
                    if lock.read_text().strip() == str(self.process.pid):
                        self._remember(lock, stat.S_ISREG)
            return self
        except BaseException:
            self.__exit__(None, None, None)
            raise
        finally:
            os.close(read_fd)
            if write_fd is not None:
                os.close(write_fd)

    def _remember(self, path, expected_type):
        info = path.lstat()
        if info.st_uid != os.getuid() or not expected_type(info.st_mode):
            raise RuntimeError("Unexpected owned display resource")
        self.paths[path] = (info.st_dev, info.st_ino, info.st_ctime_ns)

    def environment(self):
        env = os.environ.copy()
        env.update(DISPLAY=self.display, XAUTHORITY=str(self.authority))
        return env

    def __exit__(self, *_):
        if self.process is not None and self.process.poll() is None:
            # A failure fixture can leave our server suspended.
            self.process.send_signal(signal.SIGCONT)
            self.process.terminate()
            try:
                self.process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        for path, identity in self.paths.items():
            try:
                info = path.lstat()
                if (info.st_dev, info.st_ino, info.st_ctime_ns) == identity:
                    path.unlink()
            except FileNotFoundError:
                pass
        if self.temporary is not None:
            self.temporary.cleanup()
