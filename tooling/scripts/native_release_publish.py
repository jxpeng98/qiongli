#!/usr/bin/env python3
"""Publish an explicitly dispatched native release using qualified CI assets."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import tomllib

try:
    from .release_version import parse_release_version
except ImportError:
    from release_version import parse_release_version

ROOT = Path(__file__).resolve().parents[2]


def run(*args):
    return subprocess.check_output(list(map(str, args)), cwd=ROOT, text=True).strip()


def select_run(runs, commit, tag):
    matches = [item for item in runs if item['head_sha'] == commit
               and item['head_branch'] == tag and item['event'] == 'workflow_dispatch']
    return max(matches, key=lambda item: item['id']) if matches else None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tag', required=True)
    args = parser.parse_args()
    identity = parse_release_version(args.tag)
    commit = run('git', 'rev-parse', 'HEAD')
    version = tomllib.loads((ROOT / 'packages/qiongli-native/Cargo.toml').read_text())['workspace']['package']['version']
    if (os.environ.get('GITHUB_ACTIONS') != 'true' or identity.release_line != 'native-2x'
            or args.tag != identity.repo_tag
            or version != identity.version or os.environ.get('GITHUB_SHA') != commit
            or os.environ.get('GITHUB_REF') != f'refs/tags/{args.tag}'):
        parser.error('publication requires Actions at the matching immutable native release tag')
    notes = ROOT / 'tooling/release' / f'{args.tag}.md'
    if not notes.is_file():
        parser.error('reviewed release notes are required')
    repo = os.environ['GITHUB_REPOSITORY']
    remote_commit = json.loads(run('gh', 'api', f'repos/{repo}/commits/{args.tag}'))['sha']
    if remote_commit != commit:
        raise ValueError('remote tag does not identify the checked-out source')
    if not identity.is_prerelease:
        main_commit = json.loads(run('gh', 'api', f'repos/{repo}/commits/main'))['sha']
        if main_commit != commit:
            raise ValueError('stable publication requires the frozen main head')
    run('gh', 'workflow', 'run', 'native-cli-distribution.yml', '--repo', repo, '--ref', args.tag)
    deadline = time.monotonic() + 90 * 60
    while time.monotonic() < deadline:
        runs = json.loads(run('gh', 'api', f'repos/{repo}/actions/workflows/native-cli-distribution.yml/runs?head_sha={commit}&per_page=30'))['workflow_runs']
        candidate = select_run(runs, commit, args.tag)
        if candidate and candidate['status'] == 'completed':
            if candidate['conclusion'] != 'success':
                raise RuntimeError(f"Native distribution failed: {candidate['html_url']}")
            break
        time.sleep(20)
    else:
        raise TimeoutError('native distribution did not complete; nothing published')
    assets = Path(os.environ['RUNNER_TEMP']) / 'native-release-assets'
    run('gh', 'run', 'download', str(candidate['id']), '--repo', repo,
        '--name', 'cli-release-assets', '--dir', assets)
    run(sys.executable, ROOT / 'tooling/scripts/native_release_assets.py', 'verify',
        '--root', assets, '--version', args.tag, '--commit', commit, '--require-ci')
    manifest = json.loads((assets / 'release-manifest.json').read_text())
    upload_names = {'release-manifest.json', 'SHA256SUMS',
                    *(item['file'] for item in manifest['artifacts'])}
    # Refuse an existing release rather than replacing any advertised bytes.
    releases = json.loads(run('gh', 'api', f'repos/{repo}/releases?per_page=100'))
    if any(release['tag_name'] == args.tag for release in releases):
        raise RuntimeError('release already exists; inspect it before attempting recovery')
    run('gh', 'release', 'create', args.tag, '--repo', repo, '--verify-tag', '--draft',
        *(['--prerelease'] if identity.is_prerelease else []),
        '--title', f'Qiongli {args.tag}', '--notes-file', notes,
        *sorted(assets / name for name in upload_names))
    run('gh', 'release', 'edit', args.tag, '--repo', repo, '--draft=false',
        *(['--prerelease'] if identity.is_prerelease else ['--prerelease=false', '--latest']))
    public_assets = Path(os.environ['RUNNER_TEMP']) / 'native-public-assets'
    run('gh', 'release', 'download', args.tag, '--repo', repo, '--dir', public_assets)
    run(sys.executable, ROOT / 'tooling/scripts/native_release_assets.py', 'verify',
        '--root', public_assets, '--version', args.tag, '--commit', commit, '--require-ci')
    # GITHUB_TOKEN release events do not start other workflows. Dispatch the
    # existing publishers explicitly, preserving their environments and gates.
    for workflow in ('publish-npm.yml', 'publish-pypi.yml', 'publish-cargo.yml'):
        run('gh', 'workflow', 'run', workflow, '--repo', repo, '--ref', args.tag,
            '-f', 'publish_release=true')
    print(f'Published GitHub {identity.channel} release {args.tag}; dispatched npm, PyPI and Cargo uploads.')


if __name__ == '__main__':
    main()
