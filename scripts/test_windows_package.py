import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import zipfile
import struct

from verify_windows_package import verify
from windows_imports import verify_portable_runtime


def pe_fixture(dll=None):
    binary = bytearray(1024)
    binary[:2] = b"MZ"
    struct.pack_into("<I", binary, 60, 128)
    struct.pack_into("<4sHH", binary, 128, b"PE\0\0", 0x8664, 1)
    struct.pack_into("<H", binary, 148, 240)
    struct.pack_into("<H", binary, 152, 0x20b)
    struct.pack_into("<I", binary, 260, 16)
    struct.pack_into("<IIII", binary, 400, 512, 0x1000, 512, 512)
    if dll:
        struct.pack_into("<II", binary, 272, 0x1000, 40)
        struct.pack_into("<IIIII", binary, 512, 0, 0, 0, 0x1040, 0)
        binary[576:576 + len(dll) + 1] = dll.encode("ascii") + b"\0"
    return binary


class WindowsPackageTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.name = "nanoclick-0.1.0-windows-x86_64-preview"
        binary = pe_fixture()
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

    def test_external_vc_runtime_is_rejected_case_insensitively(self):
        for dll in ("VCRUNTIME140.dll", "msvcp140.dll", "vcruntime140_1.DLL"):
            with self.subTest(dll=dll), self.assertRaisesRegex(ValueError, "external VC"):
                verify_portable_runtime(pe_fixture(dll))

    def test_os_import_is_allowed_and_unreferenced_runtime_text_is_ignored(self):
        binary = pe_fixture("KERNEL32.dll")
        binary[800:816] = b"VCRUNTIME140.dll"
        self.assertEqual(verify_portable_runtime(binary), ["KERNEL32.dll"])

    def test_truncated_or_unmapped_import_metadata_is_rejected(self):
        binary = pe_fixture("KERNEL32.dll")
        for malformed in (binary[:200], binary[:540]):
            with self.assertRaises(ValueError):
                verify_portable_runtime(malformed)

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
