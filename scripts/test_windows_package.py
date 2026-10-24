import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

from verify_windows_package import verify


class WindowsPackageTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.name = "nanoclick-0.1.0-windows-x86_64-preview"
        binary = bytearray(134)
        binary[:2] = b"MZ"
        binary[60:64] = (128).to_bytes(4, "little")
        binary[128:] = b"PE\x00\x00\x64\x86"
        build = {"target": "x86_64-pc-windows-msvc", "preview": True,
                 "packaging_commit": "fixture", "unresolved_dependency_notices": [],
                 "input_binary_sha256": hashlib.sha256(binary).hexdigest()}
        with zipfile.ZipFile(self.root / (self.name + ".zip"), "w") as archive:
            for name in ("LICENSE.txt", "nanoclick.png", "README.txt", "dependency-licenses.json"):
                archive.writestr(self.name + "/" + name, b"fixture")
            archive.writestr(self.name + "/build.json", json.dumps(build))
            archive.writestr(self.name + "/nanoclick.exe", binary)
        with zipfile.ZipFile(self.root / (self.name + "-symbols.zip"), "w") as archive:
            archive.writestr(self.name + "-symbols/nanoclick.pdb", b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\x00\x00\x00fixture")
        self.manifest = self.root / (self.name + "-SHA256SUMS")
        self.manifest.write_text("".join(
            hashlib.sha256(path.read_bytes()).hexdigest() + "  " + path.name + "\n"
            for path in sorted(self.root.glob("*.zip"))))

    def test_preview_and_separate_symbols_pass(self):
        self.assertEqual(verify(self.root, "fixture")["packaging_commit"], "fixture")

    def test_corrupted_archive_is_rejected_before_reading_zip(self):
        with (self.root / (self.name + ".zip")).open("ab") as content:
            content.write(b"corruption")
        with self.assertRaisesRegex(ValueError, "Checksum mismatch"):
            verify(self.root, "fixture")

    def test_wrong_commit_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "provenance"):
            verify(self.root, "different")

    def test_missing_symbols_checksum_is_rejected(self):
        lines = self.manifest.read_text().splitlines()
        self.manifest.write_text(next(line for line in lines if "-symbols.zip" not in line) + "\n")
        with self.assertRaisesRegex(ValueError, "both required"):
            verify(self.root, "fixture")


if __name__ == "__main__":
    unittest.main()
