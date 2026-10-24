#!/usr/bin/env python3
"""Cache notices at Cargo VCS revisions or explicitly recorded release-tag revisions."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import urllib.error
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
CACHE = ROOT / 'assets' / 'third-party'
NAMES = {'accesskit', 'accesskit_atspi_common', 'accesskit_consumer', 'accesskit_unix', 'accesskit_winit', 'accesskit_windows', 'ecolor', 'eframe', 'egui', 'egui-wgpu', 'egui-winit', 'egui_glow', 'emath', 'epaint', 'epaint_default_fonts', 'gl_generator', 'khronos_api', 'profiling'}

def main():
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=ROOT))
    cache, manifest = {}, {}
    revisions = json.loads((CACHE / 'revisions.json').read_text())
    for package in metadata['packages']:
        if package['name'] not in NAMES:
            continue
        source = Path(package['manifest_path']).parent
        sha = json.loads((source / '.cargo_vcs_info.json').read_text())['git']['sha1']
        repository = urllib.parse.urlparse(package['repository'])
        if repository.hostname != 'github.com' or not re.fullmatch('[0-9a-f]{40}', sha):
            raise ValueError('Only exact public GitHub source revisions are supported.')
        owner, repo = repository.path.strip('/').split('/')[:2]
        repo = repo.removesuffix('.git')
        crate_sha = sha
        clarification = revisions.get(f'{owner}/{repo}@{package["version"]}')
        if clarification is not None:
            sha = clarification['commit']
        key = (owner, repo, sha)
        if key not in cache:
            found = []
            for filename in ('LICENSE-MIT', 'LICENSE-APACHE', 'LICENSE', 'LICENSE.md', 'LICENSE.txt'):
                url = f'https://raw.githubusercontent.com/{owner}/{repo}/{sha}/{filename}'
                try:
                    with urllib.request.urlopen(url, timeout=30) as response:
                        content = response.read(262145)
                except urllib.error.HTTPError as error:
                    if error.code == 404:
                        continue
                    raise
                if len(content) > 262144 or b'<html' in content.lower():
                    raise ValueError(f'Invalid notice response: {url}')
                relative = Path(f'{owner}-{repo}-{sha}') / filename
                output = CACHE / relative
                output.parent.mkdir(parents=True, exist_ok=True)
                output.write_bytes(content)
                found.append({'path': relative.as_posix(), 'source_url': url, 'sha256': hashlib.sha256(content).hexdigest()})
            if not found:
                raise ValueError(f'No upstream notice found at {key}')
            cache[key] = found
        manifest[f'{package["name"]}@{package["version"]}'] = {'crate_revision': crate_sha, 'notice_revision': sha, 'clarification': clarification, 'files': cache[key]}
    CACHE.mkdir(parents=True, exist_ok=True)
    (CACHE / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'Cached source notices for {len(manifest)} crate versions at {len(cache)} immutable revisions.')

if __name__ == '__main__':
    main()
