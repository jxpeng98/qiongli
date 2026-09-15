#!/usr/bin/env python3
"""Publish an explicitly dispatched native release using qualified CI assets."""
import argparse
import hashlib
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


def select_run(runs, commit, tag, after_id=0):
    matches = [item for item in runs if item['head_sha'] == commit
               and item['head_branch'] == tag and item['event'] == 'workflow_dispatch'
               and item['id'] > after_id]
    return max(matches, key=lambda item: item['id']) if matches else None


def download_candidate(repo, tag, commit, destination, *, require_ci=False):
    release = json.loads(run('gh', 'release', 'view', tag, '--repo', repo, '--json', 'isDraft,tagName'))
    if release != {'isDraft': True, 'tagName': tag}:
        raise ValueError('publication requires the reviewed draft; published releases are immutable')
    run('gh', 'release', 'download', tag, '--repo', repo, '--dir', destination)
    run(sys.executable, ROOT / 'tooling/scripts/native_release_assets.py', 'verify',
        '--root', destination, '--version', tag, '--commit', commit,
        '--require-ci' if require_ci else '--require-local-macos')
    manifest = json.loads((destination / 'release-manifest.json').read_text())
    names = {'release-manifest.json', *(item['file'] for item in manifest['artifacts'])}
    if {p.name for p in destination.iterdir()} != names | {'SHA256SUMS'}:
        raise ValueError('draft contains unverified attachments')
    checksums = ''.join(f'{hashlib.sha256((destination / name).read_bytes()).hexdigest()}  {name}\n'
                        for name in sorted(names))
    if (destination / 'SHA256SUMS').read_text() != checksums:
        raise ValueError('draft checksum list does not match the verified packet')
    return hashlib.sha256((destination / 'release-manifest.json').read_bytes()).hexdigest()


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
    assets = Path(os.environ['RUNNER_TEMP']) / 'native-release-assets'
    packet_digest = download_candidate(repo, args.tag, commit, assets)
    runs_url = f'repos/{repo}/actions/workflows/native-cli-distribution.yml/runs?head_sha={commit}&per_page=30'
    existing = json.loads(run('gh', 'api', runs_url))['workflow_runs']
    after_id = max((r['id'] for r in existing), default=0)
    run('gh', 'workflow', 'run', 'native-cli-distribution.yml', '--repo', repo, '--ref', args.tag,
        '-f', 'verify_release=true')
    deadline = time.monotonic() + 90 * 60
    while time.monotonic() < deadline:
        runs = json.loads(run('gh', 'api', runs_url))['workflow_runs']
        candidate = select_run(runs, commit, args.tag, after_id)
        if candidate and candidate['status'] == 'completed':
            if candidate['conclusion'] != 'success':
                raise RuntimeError(f"Native distribution failed: {candidate['html_url']}")
            break
        time.sleep(20)
    else:
        raise TimeoutError('native distribution did not complete; nothing published')
    final_assets = Path(os.environ['RUNNER_TEMP']) / 'native-final-assets'
    if download_candidate(repo, args.tag, commit, final_assets, require_ci=True) != packet_digest:
        raise ValueError('draft packet changed during qualification')
    if json.loads(run('gh', 'api', f'repos/{repo}/commits/{args.tag}'))['sha'] != commit:
        raise ValueError('remote tag changed during qualification')
    if not identity.is_prerelease and json.loads(run('gh', 'api', f'repos/{repo}/commits/main'))['sha'] != commit:
        raise ValueError('stable main changed during qualification')
    run('gh', 'release', 'edit', args.tag, '--repo', repo, '--draft=false',
        '--title', f'Qiongli {args.tag}', '--notes-file', notes,
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
