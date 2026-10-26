import tempfile
from pathlib import Path
import unittest
import subprocess
from package import (
    package_readme,
    bundle_name,
    publish,
    renderer_feature_args,
    tar_bundle,
    validate_renderer,
    verify_binary,
)

class PackageTests(unittest.TestCase):
    def test_wgpu_and_glow_readmes_name_only_their_renderer_requirements(self):
        wgpu = package_readme(True, "linux", "wgpu")
        glow = package_readme(True, "linux", "glow")
        self.assertIn("requires a Vulkan loader", wgpu)
        self.assertIn("libvulkan1", wgpu)
        self.assertIn("requires OpenGL/EGL", glow)
        self.assertIn("libgl1 libegl1", glow)
        self.assertNotIn("requires a Vulkan loader", glow)

    def test_glow_inventory_uses_its_actual_cargo_feature_set(self):
        self.assertEqual(renderer_feature_args("wgpu"), [])
        self.assertEqual(renderer_feature_args("glow"), ["--no-default-features", "--features", "glow"])

    def test_glow_bundle_name_does_not_collide_with_wgpu(self):
        self.assertEqual(bundle_name("0.1.0", "linux", "wgpu", True),
                         "nanoclick-0.1.0-linux-x86_64-preview")
        self.assertEqual(bundle_name("0.1.0", "linux", "glow", True),
                         "nanoclick-0.1.0-linux-x86_64-glow-preview")

    def test_windows_rejects_the_linux_only_glow_renderer(self):
        with self.assertRaisesRegex(ValueError, "Windows builds require the WGPU renderer"):
            validate_renderer("windows", "glow")
        validate_renderer("windows", "wgpu")

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
