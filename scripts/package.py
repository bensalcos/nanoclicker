#!/usr/bin/env python3
"""Build a local bundle from an existing release binary; never publishes online."""
from __future__ import annotations
import argparse
import gzip
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import time
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def run(*args: str, cwd: Path = ROOT) -> str:
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def sha256(path: Path) -> str:
    with path.open("rb") as content:
        return hashlib.file_digest(content, "sha256").hexdigest()


def verify_binary(path: Path, platform: str) -> None:
    with path.open("rb") as content:
        header = content.read(64)
        if platform == "linux":
            if header[:6] != b"\x7fELF\x02\x01" or int.from_bytes(header[18:20], "little") != 62:
                raise ValueError("Expected a Linux x86_64 ELF binary.")
        else:
            if header[:2] != b"MZ" or len(header) != 64:
                raise ValueError("Expected a Windows x86_64 PE binary.")
            content.seek(int.from_bytes(header[60:64], "little"))
            pe = content.read(6)
            if pe[:4] != b"PE\x00\x00" or int.from_bytes(pe[4:6], "little") != 0x8664:
                raise ValueError("Expected a Windows x86_64 PE binary.")


def publish(source: Path, destination: Path) -> None:
    """Atomic no-clobber publication; a failure never removes the destination."""
    os.link(source, destination)


def tar_bundle(source: Path, destination: Path, epoch: int) -> None:
    with destination.open("wb") as output:
        with gzip.GzipFile(filename="", mode="wb", fileobj=output, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w") as archive:
                for path in sorted(source.rglob("*")):
                    if not path.is_file():
                        continue
                    info = archive.gettarinfo(str(path), arcname=(Path(source.name) / path.relative_to(source)).as_posix())
                    info.uid = info.gid = 0
                    info.uname = info.gname = ""
                    info.mtime = epoch
                    info.mode = 0o755 if path.name == "nanoclick" else 0o644
                    with path.open("rb") as content:
                        archive.addfile(info, content)


def zip_bundle(source: Path, destination: Path, epoch: int) -> None:
    date = time.gmtime(max(epoch, 315532800))[:6]
    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in sorted(source.rglob("*")):
            if path.is_file():
                info = zipfile.ZipInfo((Path(source.name) / path.relative_to(source)).as_posix(), date_time=date)
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o644 << 16
                archive.writestr(info, path.read_bytes())


def renderer_feature_args(renderer: str) -> list[str]:
    return ["--no-default-features", "--features", "glow"] if renderer == "glow" else []


def validate_renderer(platform: str, renderer: str) -> None:
    if platform == "windows" and renderer != "wgpu":
        raise ValueError("Windows builds require the WGPU renderer.")


def bundle_name(version: str, platform: str, renderer: str, preview: bool) -> str:
    renderer_suffix = "-glow" if platform == "linux" and renderer == "glow" else ""
    preview_suffix = "-preview" if preview else ""
    return f"nanoclick-{version}-{platform}-x86_64{renderer_suffix}{preview_suffix}"


def package_readme(preview: bool, platform: str, renderer: str) -> str:
    title = "NanoClick local preview" if preview else "NanoClick"
    run_line = "Run ./nanoclick on Linux.\n" if platform == "linux" else "Run nanoclick.exe on Windows.\n"
    if platform == "linux" and renderer == "wgpu":
        runtime_requirements = (
            "The WGPU renderer requires a Vulkan loader, a working Vulkan driver, "
            "and X11 or Wayland runtime libraries.\n"
            "Ubuntu X11: libvulkan1 mesa-vulkan-drivers libx11-6 libxkbcommon0\n"
            "libxkbcommon-x11-0 libxi6 libxrandr2 libxcursor1 libxinerama1 (and dependencies).\n"
        )
    elif platform == "linux":
        runtime_requirements = (
            "The Glow renderer requires OpenGL/EGL and X11 or Wayland runtime libraries.\n"
            "Ubuntu X11: libgl1 libegl1 libx11-6 libxkbcommon0 libxkbcommon-x11-0\n"
            "libxi6 libxrandr2 libxcursor1 libxinerama1 (and dependencies).\n"
        )
    else:
        runtime_requirements = (
            "This x86_64 Windows build uses the WGPU renderer and the graphics APIs available in Windows.\n"
            "Extract the ZIP and run nanoclick.exe; no installer or automatic startup is included.\n"
        )
    if platform == "linux":
        runtime_requirements += "Wayland requires desktop portals and pointer consent.\n"
        install_guidance = (
            "The Linux desktop entry expects nanoclick on PATH and its icon in the icon theme.\n"
            "Install the binary into ~/.local/bin, the .desktop into ~/.local/share/applications,\n"
            "and nanoclick.png as ~/.local/share/icons/hicolor/256x256/apps/io.github.bensalcos.NanoClick.png.\n"
        )
    else:
        install_guidance = "Remove the extracted folder to uninstall the preview.\n"
    return (
        f"{title}\n\n{run_line}No automatic installation or startup occurs.\n"
        + runtime_requirements
        + "This host-built binary has not been accepted on every target OS.\n"
        + install_guidance
        + "See build.json for preview status and unresolved dependency notices.\n"
        + "Platform acceptance remains subject to docs/STATUS.md in the source checkout.\n"
    )


def notices(stage: Path, target: str, renderer: str) -> list[str]:
    feature_args = renderer_feature_args(renderer)
    metadata = json.loads(run("cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", target, *feature_args))
    tree = run("cargo", "tree", "--locked", "-p", "nanoclick", "--target", target, *feature_args,
               "--prefix", "none", "--format", "{p}", "--edges", "normal,build", "--no-dedupe")
    selected = {match.groups() for line in tree.splitlines() if (match := re.match(r"^(\S+) v(\S+)", line))}
    cached_path = ROOT / "assets" / "third-party" / "manifest.json"
    cached = json.loads(cached_path.read_text()) if cached_path.exists() else {}
    lockfile = tomllib.loads((ROOT / "Cargo.lock").read_text())
    checksums = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in lockfile["package"]}
    inventory, missing = [], []
    for package in sorted(metadata["packages"], key=lambda value: (value["name"], value["version"])):
        if (package["name"], package["version"]) not in selected or package["source"] is None:
            continue
        name = f'{package["name"]}-{package["version"]}'
        directory = Path(package["manifest_path"]).parent
        copied = []
        for path in sorted(directory.rglob("*")):
            relative = path.relative_to(directory)
            words = str(relative).upper()
            legal = any(word in words for word in ("LICENSE", "LICENCE", "COPYING", "COPYRIGHT", "NOTICE"))
            font_notice = "fonts" in relative.parts and path.suffix == ".txt"
            if not (legal or font_notice) or path.is_symlink() or not path.is_file():
                continue
            if path.stat().st_size > 2 * 1024 * 1024:
                raise ValueError(f"Unexpectedly large notice: {name}/{relative}")
            output = stage / "THIRD-PARTY-NOTICES" / name / relative
            output.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, output)
            copied.append(str(relative))
        cached_notice = cached.get(f'{package["name"]}@{package["version"]}')
        if cached_notice:
            upstream = stage / "THIRD-PARTY-NOTICES" / name / "upstream"
            upstream.mkdir(parents=True, exist_ok=True)
            for notice in cached_notice["files"]:
                source = (ROOT / "assets" / "third-party" / notice["path"]).resolve()
                if not source.is_relative_to((ROOT / "assets" / "third-party").resolve()) or sha256(source) != notice["sha256"]:
                    raise ValueError(f"Cached notice integrity failed: {name}")
                shutil.copyfile(source, upstream / source.name)
                copied.append(f"upstream/{source.name}")
            (upstream / "source.json").write_text(json.dumps(cached_notice, indent=2) + "\n")
        # Preserve complete upstream source for MPL-covered dependencies instead
        # of relying on a repository URL as the only source-availability notice.
        if "MPL-2.0" in (package["license"] or ""):
            archive = directory.parents[2] / "cache" / directory.parent.name / f"{name}.crate"
            expected = checksums[(package["name"], package["version"], package["source"]) ]
            if sha256(archive) != expected:
                raise ValueError(f"Upstream source archive checksum failed: {name}")
            output = stage / "SOURCE-ARCHIVES"
            output.mkdir(exist_ok=True)
            shutil.copyfile(archive, output / archive.name)
        inventory.append({"name": package["name"], "version": package["version"], "license": package["license"], "repository": package["repository"], "notices": copied})
        if not copied:
            missing.append(name)
    (stage / "dependency-licenses.json").write_text(json.dumps(inventory, indent=2) + "\n")
    return missing


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--platform", choices=["linux", "windows"], required=True)
    parser.add_argument("--renderer", choices=["wgpu", "glow"], default="wgpu",
                        help="renderer used to build the binary (Linux only: glow; default: wgpu)")
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "dist")
    parser.add_argument("--preview", action="store_true", help="local validation bundle with unresolved release gates")
    args = parser.parse_args()
    try:
        validate_renderer(args.platform, args.renderer)
    except ValueError as error:
        parser.error(str(error))
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    extension = ".exe" if args.platform == "windows" else ""
    binary = (args.binary or ROOT / "target" / "release" / f"nanoclick{extension}").resolve()
    verify_binary(binary, args.platform)
    project_license = (ROOT / "LICENSE.txt").read_text()
    if ("[year]" in project_license or "[fullname]" in project_license) and not args.preview:
        raise ValueError("LICENSE.txt has owner/year placeholders. Resolve them before distribution; --preview only creates a local validation bundle.")
    target = "x86_64-pc-windows-msvc" if args.platform == "windows" else "x86_64-unknown-linux-gnu"
    epoch = int(run("git", "show", "-s", "--format=%ct", "HEAD"))
    name = bundle_name(version, args.platform, args.renderer, args.preview)
    args.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".package-", dir=args.output) as temporary:
        staging = Path(temporary)
        stage, symbols = staging / name, staging / f"{name}-symbols"
        stage.mkdir()
        symbols.mkdir()
        executable = stage / f"nanoclick{extension}"
        shutil.copyfile(binary, executable)
        if args.platform == "linux":
            subprocess.run(["objcopy", "--only-keep-debug", str(binary), str(symbols / "nanoclick.debug")], check=True)
            subprocess.run(["strip", "--strip-unneeded", str(executable)], check=True)
            subprocess.run(["objcopy", f"--add-gnu-debuglink={symbols / 'nanoclick.debug'}", str(executable)], check=True)
            executable.chmod(0o755)
            shutil.copyfile(ROOT / "assets/io.github.bensalcos.NanoClick.desktop", stage / "io.github.bensalcos.NanoClick.desktop")
        else:
            pdb = binary.with_suffix(".pdb")
            if not pdb.exists():
                raise ValueError("The release PDB is missing; build with the repository's release profile.")
            shutil.copyfile(pdb, symbols / "nanoclick.pdb")
        shutil.copyfile(ROOT / "assets/nanoclick.png", stage / "nanoclick.png")
        shutil.copyfile(ROOT / "LICENSE.txt", stage / "LICENSE.txt")
        missing = notices(stage, target, args.renderer)
        if missing and not args.preview:
            raise ValueError("Dependency notices need review: " + ", ".join(missing))
        build = {"version": version, "target": target, "renderer": args.renderer, "packaging_commit": run("git", "rev-parse", "HEAD"), "packaging_tree_dirty": bool(run("git", "status", "--porcelain")), "input_binary_sha256": sha256(binary), "binary_build_commit": "Not inferred from the packaging checkout", "preview": args.preview, "native_acceptance": "See docs/STATUS.md; packaging does not certify acceptance.", "unresolved_dependency_notices": missing}
        (stage / "build.json").write_text(json.dumps(build, indent=2) + "\n")
        (stage / "README.txt").write_text(package_readme(args.preview, args.platform, args.renderer))
        suffix = ".zip" if args.platform == "windows" else ".tar.gz"
        archive = zip_bundle if args.platform == "windows" else tar_bundle
        outputs = []
        for folder in (stage, symbols):
            output = staging / (folder.name + suffix)
            archive(folder, output, epoch)
            outputs.append(output)
        digest = staging / f"{name}-SHA256SUMS"
        digest.write_text("".join(f"{sha256(path)}  {path.name}\n" for path in outputs))
        outputs.append(digest)
        # Publish only complete files. If a later sidecar conflicts, existing
        # published files remain intact; use a fresh output directory to retry.
        for output in outputs:
            publish(output, args.output / output.name)
        print(json.dumps({"output": str(args.output), "files": [p.name for p in outputs], "preview": args.preview, "missing_dependency_notices": len(missing)}))


if __name__ == "__main__":
    main()
