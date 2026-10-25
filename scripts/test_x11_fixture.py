"""Real private-Xvfb checks; no input injection and no host display access."""
from concurrent.futures import ThreadPoolExecutor
import importlib.util
from pathlib import Path
import socket
import struct
import threading
import unittest
from unittest import mock

import x11_fixture

from x11_fixture import OwnedX11, STARTUP_TIMEOUT

spec = importlib.util.spec_from_file_location("x11_stall_observer", Path(__file__).with_name("verify-x11-stall.py"))
stall = importlib.util.module_from_spec(spec)
spec.loader.exec_module(stall)


class WorkerObservationTests(unittest.TestCase):
    def test_disappearing_thread_does_not_hide_live_blocked_worker(self):
        for vanished in (FileNotFoundError, ProcessLookupError):
            for field in ("comm", "wchan"):
                with self.subTest(error=vanished.__name__, field=field):
                    departing, live = mock.MagicMock(), mock.MagicMock()
                    for task in (departing, live):
                        files = {"comm": mock.Mock(), "wchan": mock.Mock()}
                        files["comm"].read_text.return_value = "nanoclick-worker"
                        files["wchan"].read_text.return_value = "do_poll"
                        if task is departing:
                            files[field].read_text.side_effect = vanished("Thread exited")
                        task.__truediv__.side_effect = files.__getitem__
                    root = mock.Mock()
                    root.glob.return_value = [departing, live]
                    with mock.patch.object(stall, "Path", return_value=root):
                        self.assertEqual(stall.blocked_poll(mock.Mock(pid=123), "nanoclick-work"), "do_poll")

    def test_permission_errors_remain_failures(self):
        task = mock.MagicMock()
        task.__truediv__.return_value.read_text.side_effect = PermissionError("Observation denied")
        root = mock.Mock()
        root.glob.return_value = [task]
        with mock.patch.object(stall, "Path", return_value=root):
            with self.assertRaises(PermissionError):
                stall.blocked_poll(mock.Mock(pid=123), "nanoclick-work")


class OwnedDisplayTests(unittest.TestCase):
    def test_empty_readiness_observation_keeps_waiting_for_same_server(self):
        selector_type = x11_fixture.selectors.DefaultSelector

        class EmptyFirstObservation:
            def __init__(self):
                self.selector = selector_type()
                self.first = True

            def __enter__(self):
                return self

            def __exit__(self, *args):
                return self.selector.__exit__(*args)

            def register(self, *args):
                return self.selector.register(*args)

            def select(self, timeout):
                if self.first:
                    self.first = False
                    return []
                return self.selector.select(timeout)

        with mock.patch.object(x11_fixture.selectors, "DefaultSelector", EmptyFirstObservation):
            with OwnedX11("640x480x24") as fixture:
                self.assertIsNone(fixture.process.poll())

    def test_startup_failure_retains_server_diagnostics(self):
        popen = x11_fixture.subprocess.Popen

        def invalid_server(arguments, **kwargs):
            return popen([*arguments, "-nanoclick-invalid-fixture-option"], **kwargs)

        with mock.patch.object(x11_fixture.subprocess, "Popen", invalid_server):
            with self.assertRaisesRegex(RuntimeError, "nanoclick-invalid-fixture-option"):
                with OwnedX11("640x480x24"):
                    self.fail("The invalid Xvfb invocation must not become usable")

    def test_concurrent_servers_have_distinct_verified_displays(self):
        barrier = threading.Barrier(2, timeout=STARTUP_TIMEOUT + 5)

        def own():
            try:
                with OwnedX11("640x480x24") as fixture:
                    process = fixture.process
                    barrier.wait()
                    self.assertIsNone(process.poll())
                    identity = fixture.display, process.pid
                self.assertIsNotNone(process.poll())
                return identity
            except BaseException:
                barrier.abort()
                raise

        with ThreadPoolExecutor(max_workers=2) as executor:
            futures = [executor.submit(own) for _ in range(2)]
            results, errors = [], []
            for future in futures:
                try:
                    results.append(future.result())
                except Exception as error:
                    errors.append(error)
            if errors:
                raise ExceptionGroup("Owned Xvfb startup failures", errors)
            first, second = results
        self.assertNotEqual(first[0], second[0])
        self.assertNotEqual(first[1], second[1])

    def test_server_requires_its_private_cookie(self):
        with OwnedX11("640x480x24") as fixture:
            cookie = fixture.authority.read_bytes()[-32:]
            address = f"/tmp/.X11-unix/X{fixture.display[1:]}"

            def setup(data):
                name = b"MIT-MAGIC-COOKIE-1" if data else b""
                request = struct.pack("<BBHHHHH", ord("l"), 0, 11, 0, len(name), len(data), 0)
                request += name + bytes((-len(name)) % 4)
                request += data + bytes((-len(data)) % 4)
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
                    client.settimeout(2)
                    client.connect(address)
                    client.sendall(request)
                    return client.recv(1)

            self.assertEqual(setup(cookie), b"\x01")
            self.assertEqual(setup(b""), b"\x00")
            wrong = bytes([cookie[0] ^ 1]) + cookie[1:]
            self.assertEqual(setup(wrong), b"\x00")
            self.assertEqual(Path(f"/proc/{fixture.process.pid}/exe").resolve().name, "Xvfb")


if __name__ == "__main__":
    unittest.main()
