import tempfile
from pathlib import Path
import unittest
import subprocess
from package import publish, tar_bundle, verify_binary

class PackageTests(unittest.TestCase):
    def test_cached_notice_bytes_survive_windows_checkout_filters(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            (root / ".gitattributes").write_bytes((Path(__file__).resolve().parents[1] / ".gitattributes").read_bytes())
            notice = root / "assets/third-party/example/LICENSE-MIT"
            notice.parent.mkdir(parents=True)
            original = b"Copyright fixture\nPermission notice\n"
            notice.write_bytes(original)
            subprocess.run(["git", "-C", str(root), "-c", "core.autocrlf=false", "add", "."], check=True)
            filtered = subprocess.check_output(["git", "-C", str(root), "-c", "core.autocrlf=true", "cat-file", "--filters", ":assets/third-party/example/LICENSE-MIT"])
            self.assertEqual(filtered, original)

    def test_existing_publication_is_never_overwritten_or_removed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            original, candidate = root / "bundle", root / "candidate"
            original.write_bytes(b"existing valid artifact")
            candidate.write_bytes(b"replacement")
            with self.assertRaises(FileExistsError):
                publish(candidate, original)
            self.assertEqual(original.read_bytes(), b"existing valid artifact")
            self.assertEqual(candidate.read_bytes(), b"replacement")

    def test_rejects_wrong_architecture_before_packaging(self):
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "binary"
            elf = bytearray(64)
            elf[:6] = b"\x7fELF\x02\x01"
            elf[18:20] = (183).to_bytes(2, "little")
            binary.write_bytes(elf)
            with self.assertRaises(ValueError):
                verify_binary(binary, "linux")
            elf[18:20] = (62).to_bytes(2, "little")
            binary.write_bytes(elf)
            verify_binary(binary, "linux")

    def test_archive_is_independent_of_source_mtime(self):
        import os
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "bundle"
            source.mkdir()
            binary = source / "nanoclick"
            binary.write_bytes(b"fixed content")
            tar_bundle(source, root / "one.gz", 1234567890)
            os.utime(binary, (1600000000, 1600000000))
            tar_bundle(source, root / "two.gz", 1234567890)
            self.assertEqual((root / "one.gz").read_bytes(), (root / "two.gz").read_bytes())

if __name__ == "__main__":
    unittest.main()
