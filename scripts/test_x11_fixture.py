"""Real private-Xvfb checks; no input injection and no host display access."""
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import socket
import struct
import threading
import unittest

from x11_fixture import OwnedX11


class OwnedDisplayTests(unittest.TestCase):
    def test_concurrent_servers_have_distinct_verified_displays(self):
        barrier = threading.Barrier(2, timeout=5)

        def own():
            with OwnedX11("640x480x24") as fixture:
                process = fixture.process
                barrier.wait()
                self.assertIsNone(process.poll())
                identity = fixture.display, process.pid
            self.assertIsNotNone(process.poll())
            return identity

        with ThreadPoolExecutor(max_workers=2) as executor:
            first, second = list(executor.map(lambda _: own(), range(2)))
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
