#!/usr/bin/env python3
"""Verify preview ZIP contents without executing desktop input or extracting files."""
import argparse
import hashlib
import json
from pathlib import Path
import tempfile
import zipfile

from package import verify_binary


def verify(directory: Path, commit: str) -> dict:
    manifests = list(directory.glob("*-windows-x86_64-preview-SHA256SUMS"))
    if len(manifests) != 1:
        raise ValueError("Expected exactly one Windows preview checksum manifest.")
    prefix = manifests[0].name.removesuffix("-SHA256SUMS")
    expected = {prefix + ".zip", prefix + "-symbols.zip"}
    entries = {}
    for line in manifests[0].read_text().splitlines():
        digest, name = line.split("  ", 1)
        if name not in expected or name in entries:
            raise ValueError("Unexpected or duplicate checksum entry.")
        entries[name] = digest
    if entries.keys() != expected:
        raise ValueError("Executable and symbols checksums are both required.")
    for name, digest in entries.items():
        with (directory / name).open("rb") as content:
            if hashlib.file_digest(content, "sha256").hexdigest() != digest:
                raise ValueError(f"Checksum mismatch: {name}")
    with zipfile.ZipFile(directory / (prefix + ".zip")) as archive:
        root = prefix + "/"
        for required in ("nanoclick.exe", "build.json", "LICENSE.txt", "nanoclick.png", "README.txt", "dependency-licenses.json"):
            if root + required not in archive.namelist():
                raise ValueError(f"Missing bundle file: {required}")
        build = json.loads(archive.read(root + "build.json"))
        if (build["target"] != "x86_64-pc-windows-msvc" or not build["preview"]
                or build["packaging_commit"] != commit or build["unresolved_dependency_notices"]):
            raise ValueError("Unexpected build provenance or unresolved dependency notices.")
        binary = archive.read(root + "nanoclick.exe")
        if hashlib.sha256(binary).hexdigest() != build["input_binary_sha256"]:
            raise ValueError("Packaged executable differs from the build input.")
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "nanoclick.exe"
            path.write_bytes(binary)
            verify_binary(path, "windows")
    with zipfile.ZipFile(directory / (prefix + "-symbols.zip")) as archive:
        pdb = archive.read(prefix + "-symbols/nanoclick.pdb")
        if not pdb.startswith(b"Microsoft C/C++ MSF 7.00\r\n\x1aDS\x00\x00\x00"):
            raise ValueError("Missing or invalid MSVC PDB signature.")
    return {"packaging_commit": commit, "files": entries, "scope": "Preview structure, checksums, PE architecture and PDB signature; no desktop acceptance."}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--commit", required=True)
    args = parser.parse_args()
    print(json.dumps(verify(args.directory, args.commit), indent=2))
