from __future__ import annotations

import io
import json
from pathlib import Path
import subprocess
import struct
import tarfile
import tempfile
import unittest
from unittest.mock import patch

from tooling.scripts import native_marketplace_plugins as plugins


VERSION = '2.0.0-alpha.8'
COMMIT = 'a' * 40
TARGET = 'aarch64-apple-darwin'
BINARIES = {
    TARGET: bytes.fromhex('cffaedfe0c000001') + bytes(100),
    'x86_64-unknown-linux-gnu': b'\x7fELF\x02\x01' + bytes(12) + b'\x3e\x00' + bytes(80),
    'x86_64-pc-windows-msvc': b'MZ' + bytes(58) + (64).to_bytes(4, 'little') + b'PE\x00\x00\x64\x86',
    'aarch64-unknown-linux-gnu': b'\x7fELF\x02\x01' + bytes(12) + b'\xb7\x00' + bytes(80),
}


class NativeMarketplacePluginsTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = self.root / 'content'
        self.source.mkdir()
        self.content = {
            'workflow/SKILL.md': b'---\nname: qiongli\n---\nUse supplied evidence; the Host owns models.\n',
            'workflow/references/platform-routing.md': b'Unavailable tools confer no execution evidence.\n',
            'skills/A_framing/question-refiner.md': 'Canonical evidence 中文.\n'.encode(),
        }
        for platform in plugins.PLATFORMS:
            self.content[f'.{platform}-plugin/plugin.json'] = plugins.json_bytes({
                'name': 'qiongli', 'version': VERSION, 'skills': './',
            })
        self.write_source()
        self.binary = self.root / 'qiongli'
        self.binary.write_bytes(BINARIES[TARGET])

    def write_source(self):
        self.metadata = {
            'schema_version': 1, 'version': VERSION, 'source_commit': COMMIT,
            'content_source_commit': 'b' * 40, 'pack_sha256': 'c' * 64,
            'content_root_sha256': 'd' * 64,
            'entries': [{'path': name, 'size_bytes': len(data), 'sha256': plugins.digest(data)}
                        for name, data in self.content.items()],
        }
        for name, data in self.content.items():
            path = self.source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        offset, entries = 0, []
        for entry in self.metadata['entries']:
            entries.append(dict(entry, payload_offset=offset, resource_kind='workflow', mode='regular'))
            offset += entry['size_bytes']
        manifest = dict(format_version=1, content_version=VERSION, source_commit='b' * 40,
                        content_root_sha256='d' * 64, entries=entries)
        raw = json.dumps(manifest, separators=(',', ':'), sort_keys=True).encode()
        self.metadata['pack_manifest_json'] = raw.decode()
        pack = b'QLPACK\0\0' + struct.pack('<IQ', 1, len(raw)) + raw + b''.join(self.content.values())
        self.metadata['pack_sha256'] = plugins.digest(pack)
        self.write_metadata()

    def add_workflows(self):
        source = Path(__file__).resolve().parents[1] / 'content/workflow'
        for path in [source / 'references/codex-workflow-wrapper.md', source / 'references/skill-descriptions.json', source / 'no-qiongli/SKILL.md',
                     *sorted((source / 'workflows').glob('*.md'))]:
            self.content['workflow/' + path.relative_to(source).as_posix()] = path.read_bytes()
        self.write_source()

    def write_metadata(self):
        (self.source / plugins.EXPORT).write_bytes(plugins.json_bytes(self.metadata))

    def build(self, name='out', target=TARGET):
        return plugins.build_plugins(self.source, self.root / name, VERSION, COMMIT, self.binary, target)

    def test_npm_pack_contains_source_bound_deepseek_provider_and_platform_dispatch(self):
        from tooling.scripts.native_registry_packages import npm_package
        with patch(__name__ + '.VERSION', '2.1.0'):
            self.add_workflows()
            self.content['workflow/SKILL.md'] = (Path(__file__).resolve().parents[1] / 'content/workflow/SKILL.md').read_bytes()
            for host in plugins.PLATFORMS:
                self.content[f'.{host}-plugin/plugin.json'] = plugins.json_bytes({
                    'name': 'qiongli', 'version': VERSION, 'skills': './',
                })
            self.write_source()
            binaries = {}
            for target, data in BINARIES.items():
                path = self.root / target
                path.write_bytes(data)
                binaries[target] = path
            with self.assertRaisesRegex(ValueError, 'source-bound Plugin content'):
                npm_package(self.root / 'missing', binaries, VERSION)
            packed = npm_package(self.root / 'npm-pack', binaries, VERSION,
                                 plugin_content=(self.metadata, self.content))
            with tarfile.open(packed) as archive:
                metadata = json.load(archive.extractfile('package/package.json'))
                files = {m.name.removeprefix('package/dsh/'): archive.extractfile(m).read()
                         for m in archive if m.name.startswith('package/dsh/')}
                archive.extractall(self.root / 'installed', filter='data')
            self.assertEqual(metadata['main'], 'dsh/index.mjs')
            self.assertEqual(metadata['dsh'], {'bundle': {'patch': './dsh/cordis.patch.yml'}})
            self.assertEqual(plugins.verify_deepseek_npm(files, VERSION, COMMIT, BINARIES),
                             {'pack_sha256': self.metadata['pack_sha256'], 'skills': 22})
            from tooling.scripts.native_registry_install_check import check_deepseek_npm
            import os
            with patch('tooling.scripts.native_registry_install_check.check_cli', return_value={
                    'mcp_tools': {'full': 32}, 'content_pack_sha256': self.metadata['pack_sha256']}):
                observed = check_deepseek_npm((self.root / 'installed/package').resolve(), 'node',
                    version=VERSION, root=self.root, env=os.environ.copy())
            self.assertEqual(observed['skills'], 22)
            self.assertEqual(observed['mcp_tools'], 32)
            profile_directory = self.root / 'profile # language'
            profile_directory.mkdir()
            for target, (os_name, arch, executable) in plugins.TARGETS.items():
                result = subprocess.run(['node', '--input-type=module', '-e', '''
import assert from 'node:assert/strict';
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
Object.defineProperty(process, 'platform', {value: process.argv[2]});
Object.defineProperty(process, 'arch', {value: process.argv[3]});
const { apply } = await import(pathToFileURL(process.argv[1]));
let provider, command;
await apply({skills:{registerProvider(factory) {provider = factory();}},
  provide(name, value) {command = value.command;}});
assert.ok(command.endsWith(process.argv[4]));
assert.equal((await provider.list()).length, 22);
const cjk = /[\u4e00-\u9fff]/;
for (const language of ['zh', 'en']) {
  await apply({skills:{registerProvider(factory) {provider = factory();}}, provide() {}}, {language});
  for (const entry of await provider.list()) {
    assert.equal(cjk.test(entry.description), language === 'zh');
    const loaded = await provider.get(entry);
    assert.equal(JSON.parse(loaded.content.match(/^description: (.*)$/m)[1]), entry.description);
    assert.ok(loaded.content.includes('name: ' + entry.name));
  }
}
await assert.rejects(apply({skills:{registerProvider() {}}, provide() {}}, {language:'fr'}));
const ctx = {profileContext:{dir:process.argv[5]}, skills:{registerProvider(factory) {provider = factory();}}, provide() {}};
await writeFile(join(process.argv[5], '.qiongli-skill-language.json'), JSON.stringify({language:'zh'}));
await apply(ctx);
assert.ok((await provider.list()).every(e => cjk.test(e.description)));
await writeFile(join(process.argv[5], '.qiongli-skill-language.json'), '{}');
await assert.rejects(apply(ctx));

await assert.rejects(provider.get({name:'../../private'}));
''', str(self.root / 'installed/package/dsh/index.mjs'), os_name, arch,
                    '/native/' + target + '/' + executable, str(profile_directory)], capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
            codex = self.build('source-archive')[0]
            self.assertEqual(plugins.read_plugin_content(codex, VERSION, COMMIT),
                             (self.metadata, self.content))
            files['skills.json'] = b'[]'
            with self.assertRaisesRegex(ValueError, 'native projection'):
                plugins.verify_deepseek_npm(files, VERSION, COMMIT, BINARIES)

    def test_deepseek_bundle_uses_canonical_skills_and_native_full_mcp(self):
        self.add_workflows()
        source = Path(__file__).resolve().parents[1] / 'content/workflow'
        for name in ('SKILL.md', 'no-qiongli/SKILL.md'):
            self.content['workflow/' + name] = (source / name).read_bytes()
        self.write_source()
        archives = plugins.build_plugins(self.source, self.root / 'dsh', VERSION, COMMIT,
                                        self.binary, TARGET, platforms=('deepseek',))
        self.assertEqual(len(archives), 1)
        verified = plugins.verify_archive(archives[0], VERSION, COMMIT)
        self.assertEqual(verified['platform'], 'deepseek')
        root = self.root / 'dsh/deepseek/plugins' / plugins.plugin_name(TARGET, VERSION)
        manifest = json.loads((root / 'package.json').read_bytes())
        self.assertEqual(manifest['dsh']['bundle']['patch'], './cordis.patch.yml')
        self.assertNotIn('scripts', manifest)
        patch_text = (root / 'cordis.patch.yml').read_text()
        self.assertIn('args: [mcp, serve, --profile, full, --transport, stdio]', patch_text)
        self.assertNotIn('model', patch_text)
        # Execute the actual generated module with a minimal public Cordis service surface.
        result = subprocess.run(['node', '--input-type=module', '-e', '''
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
const { apply } = await import(pathToFileURL(process.argv[1]));
let provider, command;
await apply({skills:{registerProvider(factory) { provider = factory(); }},
  provide(name, value) { assert.equal(name, 'qiongliBundle'); command = value.command; }});
assert.ok(command.endsWith('/bin/qiongli'));
const entries = await provider.list();
assert.deepEqual(entries.map(e => e.name).sort(), JSON.parse(process.argv[2]).sort());
for (const entry of entries) {
  const skill = await provider.get(entry);
  assert.ok(skill.content.includes('name: ' + entry.name));
  assert.equal(skill.resourceBase.kind, 'directory');
  assert.deepEqual(skill.invocation, {modelInvocable:true,userInvocable:true});
}
await assert.rejects(provider.get({name:'../../private'}));
''', str(root / 'index.mjs'), json.dumps(['qiongli', 'no-qiongli',
     *sorted('qiongli-' + Path(p).stem for p in self.content
             if p.startswith('workflow/workflows/') and Path(p).stem != 'qiongli')])],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        codex = plugins.project(self.content, 'codex', VERSION, TARGET, BINARIES[TARGET])
        dsh_entries = {p.relative_to(root).as_posix(): p.read_bytes() for p in root.glob('skills/*/SKILL.md')}
        self.assertEqual(dsh_entries, {p: data for p, data in codex.items()
                                     if p.startswith('skills/') and p.count('/') == 2 and p.endswith('/SKILL.md')})
        for target, binary in BINARIES.items():
            projected = plugins.project(self.content, 'deepseek', VERSION, target, binary)
            self.assertIn(plugins.binary_path(target), projected)
            self.assertIn(plugins.binary_path(target).encode(), projected['index.mjs'])
        with self.assertRaises(ValueError):
            plugins.project(self.content, 'deepseek', VERSION)
        with self.assertRaisesRegex(ValueError, 'projection collision'):
            plugins.project(dict(self.content, **{'SKILL.md': b'collision'}),
                            'deepseek', VERSION, TARGET, BINARIES[TARGET])
        with self.assertRaises(ValueError):
            plugins.build_plugins(self.source, self.root / 'duplicate', VERSION, COMMIT,
                                  self.binary, TARGET, platforms=('deepseek', 'deepseek'))
        def tamper(rows):
            index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith('/index.mjs'))
            member, data = rows[index]
            changed = data + b'\n// modified loader\n'
            rows[index] = member, changed
            index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith(plugins.RECEIPT))
            member, data = rows[index]
            receipt = json.loads(data)
            receipt['files']['index.mjs'] = {'size_bytes': len(changed), 'sha256': plugins.digest(changed)}
            rows[index] = member, plugins.json_bytes(receipt)
        self.rewrite_archive(archives[0], tamper)
        with self.assertRaisesRegex(ValueError, 'native projection'):
            plugins.verify_archive(archives[0], VERSION, COMMIT)

    def test_deepseek_legacy_two_entry_archive_remains_verifiable(self):
        self.add_workflows()
        self.content['workflow/SKILL.md'] = b'---\nname: qiongli\ndescription: Research\n---\n'
        self.write_source()
        archive = plugins.build_plugins(self.source, self.root / 'legacy-dsh', VERSION, COMMIT,
                                        self.binary, TARGET, platforms=('deepseek',))[0]
        legacy = plugins.project_deepseek(self.content, VERSION, TARGET, BINARIES[TARGET],
                                         workflow_entries=False)
        def mutate(rows):
            prefix = archive.name.removesuffix('.tar.gz') + '/plugins/' + plugins.plugin_name(TARGET, VERSION) + '/'
            kept = []
            for member, data in rows:
                name = member.name.removeprefix(prefix)
                if name == plugins.RECEIPT:
                    receipt = json.loads(data)
                    receipt['schema_version'] = 3
                    receipt['files'] = {n: {'size_bytes': len(d), 'sha256': plugins.digest(d)}
                                        for n, d in legacy.items()}
                    kept.append((member, plugins.json_bytes(receipt)))
                elif name in legacy:
                    kept.append((member, legacy[name]))
            rows[:] = kept
        self.rewrite_archive(archive, mutate)
        self.assertEqual(plugins.verify_archive(archive, VERSION, COMMIT)['platform'], 'deepseek')
        self.assertEqual([e['name'] for e in json.loads(legacy['skills.json'])], ['qiongli', 'no-qiongli'])

    def test_both_archives_preserve_canonical_bytes_and_bundle_runtime(self):
        canonical = Path(__file__).resolve().parents[1] / 'content'
        for relative in ('skills/Z_cross_cutting/model-collaborator.md',
                         'templates/agent-handoff.md', 'templates/agent-review-packet.md',
                         'workflow/references/academic-graph-continuity.md'):
            self.content[relative] = (canonical / relative).read_bytes()
        self.write_source()
        archives = self.build()
        self.assertEqual([p.name for p in archives], [plugins.archive_name(p, VERSION, TARGET) for p in plugins.PLATFORMS])
        for platform, archive in zip(plugins.PLATFORMS, archives):
            verified = plugins.verify_archive(archive, VERSION, COMMIT)
            self.assertEqual(verified['pack_sha256'], self.metadata['pack_sha256'])
            root = self.root / 'out' / platform / 'plugins' / plugins.plugin_name(TARGET, VERSION)
            for name, data in self.content.items():
                if name.startswith(('.codex-plugin/', '.claude-plugin/')):
                    continue
                self.assertEqual((root / plugins.SKILL_ROOT / name.removeprefix('workflow/')).read_bytes(), data)
            self.assertFalse((root / plugins.SKILL_ROOT / plugins.EXPORT).exists())
            manifest = json.loads((root / f'.{platform}-plugin/plugin.json').read_text())
            self.assertEqual(manifest['name'], plugins.plugin_name(TARGET, VERSION))
            self.assertEqual(manifest['skills'], './skills/')
            self.assertEqual(manifest['mcpServers'], './.mcp.json')
            mcp = json.loads((root / '.mcp.json').read_text())['mcpServers']['qiongli-next']
            self.assertEqual(mcp, plugins.mcp_manifest(platform, TARGET)['mcpServers']['qiongli-next'])
            self.assertFalse((root / plugins.BRIDGE).exists())
            self.assertEqual((root / plugins.binary_path(TARGET)).read_bytes(), BINARIES[TARGET])
            self.assertEqual(verified['binary_sha256'], plugins.digest(BINARIES[TARGET]))
        again = self.build('again')
        self.assertEqual([p.read_bytes() for p in archives], [p.read_bytes() for p in again])

    def test_reject_noncanonical_version_or_unpinned_source(self):
        for version in ['2.0.0a8', 'v2.0.0-alpha.8', '1.19.0-beta.1', '2.0.0', '2.0.0-alpha.8;echo bad']:
            with self.subTest(version=version), self.assertRaises(ValueError):
                plugins.build_plugins(self.source, self.root / 'bad', version, COMMIT, self.binary, TARGET)
        for commit in ['main', 'a' * 39, 'A' * 40, 'b' * 40]:
            with self.subTest(commit=commit), self.assertRaises(ValueError):
                plugins.build_plugins(self.source, self.root / 'bad', VERSION, commit, self.binary, TARGET)
        self.assertFalse((self.root / 'bad').exists())

    def test_stable_archives_use_stable_plugin_identity_and_verify_version(self):
        with patch(f'{__name__}.VERSION', '2.0.0'):
            for platform in plugins.PLATFORMS:
                self.content[f'.{platform}-plugin/plugin.json'] = plugins.json_bytes({
                    'name': 'qiongli', 'version': VERSION, 'interface': {},
                })
            self.write_source()
            for platform, archive in zip(plugins.PLATFORMS, self.build()):
                verified = plugins.verify_archive(archive, VERSION, COMMIT)
                self.assertEqual(verified['version'], '2.0.0')
                root = self.root / 'out' / platform / 'plugins' / plugins.plugin_name(TARGET, VERSION)
                manifest = json.loads((root / f'.{platform}-plugin/plugin.json').read_text())
                self.assertEqual(manifest['name'], 'qiongli-macos-arm64')
                self.assertEqual(set(json.loads((root / '.mcp.json').read_bytes())['mcpServers']), {'qiongli'})
                self.assertTrue(archive.name.startswith('qiongli-' + platform + '-plugin-'))
                self.assertEqual(manifest['interface']['displayName'], 'Qiongli (macos-arm64)')
                self.assertFalse((root / plugins.BRIDGE).exists())
                with self.assertRaises(ValueError):
                    plugins.verify_archive(archive, '2.0.0-beta.6', COMMIT)

    def legacy_archive(self, out, platform, target):
        files = plugins.project(self.content, platform, VERSION, target, BINARIES[target], legacy_identity=True)
        files[plugins.RECEIPT] = plugins.json_bytes({
            'schema_version': 2, 'platform': platform, 'target': target, 'source': self.metadata,
            'source_manifest_bytes': {n: self.content[n].decode() for n in
                                      ('.codex-plugin/plugin.json', '.claude-plugin/plugin.json')},
            'files': {n: {'size_bytes': len(d), 'sha256': plugins.digest(d)} for n, d in files.items()},
        })
        archive = out / plugins.archive_name(platform, VERSION, target, legacy_identity=True)
        prefix = archive.name.removesuffix('.tar.gz') + '/plugins/' + plugins.plugin_name(target, VERSION, legacy_identity=True) + '/'
        with tarfile.open(archive, 'w:gz') as packet:
            for name, data in files.items():
                member = tarfile.TarInfo(prefix + name)
                member.size, member.mode = len(data), plugins.file_mode(name, target)
                packet.addfile(member, io.BytesIO(data))
        return archive

    def test_legacy_stable_archives_remain_verifiable_and_cannot_claim_new_identity(self):
        from tooling.scripts.native_release_assets import marketplace_index
        with patch(f'{__name__}.VERSION', '2.0.0'):
            for platform in plugins.PLATFORMS:
                self.content[f'.{platform}-plugin/plugin.json'] = plugins.json_bytes({
                    'name': 'qiongli', 'version': VERSION, 'interface': {},
                })
            self.write_source()
            for platform in plugins.PLATFORMS:
                archive = self.legacy_archive(self.root, platform, TARGET)
                verified = plugins.verify_archive(archive, VERSION, COMMIT)
                self.assertEqual(verified['plugin_name'], 'qiongli-next-macos-arm64')
                index = marketplace_index(VERSION, COMMIT, [verified])['plugins'][0]
                self.assertEqual(index['name'], 'qiongli-next-macos-arm64')
                self.assertEqual(index['artifact'], archive.name)
                self.assertEqual(plugins.find_archive({archive.name: archive}, platform, VERSION, TARGET), archive)
                renamed = self.root / plugins.archive_name(platform, VERSION, TARGET)
                renamed.write_bytes(archive.read_bytes())
                with self.assertRaises(ValueError):
                    plugins.verify_archive(renamed, VERSION, COMMIT)
                with self.assertRaisesRegex(ValueError, 'duplicate'):
                    plugins.find_archive({archive.name: archive, renamed.name: renamed}, platform, VERSION, TARGET)

    def test_workflow_entries_share_canonical_instructions_and_only_codex_exposes_them(self):
        self.add_workflows()
        self.build()
        slugs = {Path(p).stem for p in self.content if p.startswith('workflow/workflows/')} - {'qiongli'}
        for platform in plugins.PLATFORMS:
            root = self.root / 'out' / platform / 'plugins' / plugins.plugin_name(TARGET, VERSION)
            entries = {p.relative_to(root).as_posix() for p in root.glob('skills/*/SKILL.md')}
            expected = {plugins.SKILL_ROOT + 'SKILL.md', 'skills/no-qiongli/SKILL.md'}
            self.assertEqual((root / 'skills/no-qiongli/SKILL.md').read_bytes(),
                             self.content['workflow/no-qiongli/SKILL.md'])
            if platform == 'codex':
                expected |= {f'skills/qiongli-{slug}/SKILL.md' for slug in slugs}
            self.assertEqual(entries, expected)
            for slug in slugs if platform == 'codex' else []:
                entry = root / f'skills/qiongli-{slug}/SKILL.md'
                text = entry.read_text()
                self.assertIn(f'name: qiongli-{slug}\n', text)
                self.assertIn(self.content[f'workflow/workflows/{slug}.md'].decode().split('\n')[1], text)
                self.assertNotIn('{{', text)
                for reference in ['SKILL.md', f'workflows/{slug}.md']:
                    link = '../qiongli-workflow/' + reference
                    self.assertIn(link, text)
                    self.assertTrue((entry.parent / link).is_file())
                self.assertLess(len(text), 2000)
        again = self.build('again')
        self.assertEqual((self.root / 'out' / again[0].name).read_bytes(), again[0].read_bytes())

    def test_no_qiongli_entry_is_source_bound_in_both_hosts(self):
        self.add_workflows()
        for archive in self.build():
            original = archive.read_bytes()
            for replacement in (None, b'Run tools before answering.'):
                archive.write_bytes(original)
                def mutate(rows):
                    index = next(i for i, (m, _) in enumerate(rows)
                                 if m.name.endswith('/skills/no-qiongli/SKILL.md'))
                    member, _ = rows.pop(index)
                    if replacement is not None:
                        rows.append((member, replacement))
                    index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith(plugins.RECEIPT))
                    member, data = rows[index]
                    receipt = json.loads(data)
                    name = 'skills/no-qiongli/SKILL.md'
                    receipt['files'].pop(name)
                    if replacement is not None:
                        receipt['files'][name] = {'size_bytes': len(replacement),
                                                  'sha256': plugins.digest(replacement)}
                    rows[index] = member, plugins.json_bytes(receipt)
                self.rewrite_archive(archive, mutate)
                with self.assertRaises(ValueError):
                    plugins.verify_archive(archive, VERSION, COMMIT)

    def test_changed_or_missing_wrapper_is_rejected_even_with_updated_receipt(self):
        self.add_workflows()
        self.content['workflow/SKILL.md'] = b'---\nname: qiongli\ndescription: Research\n---\n'
        self.write_source()
        archives = plugins.build_plugins(self.source, self.root / 'wrappers', VERSION, COMMIT,
                                        self.binary, TARGET, platforms=('codex', 'deepseek'))
        for archive in archives:
            original = archive.read_bytes()
            names = ['skills/qiongli-paper-read/SKILL.md']
            if 'deepseek' in archive.name:
                names.append('skills.json')
            for name in names:
                for replacement in [None, b'---\nname: substituted\n---\nSkip the shared workflow.\n']:
                    archive.write_bytes(original)
                    def mutate(rows):
                        index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith('/' + name))
                        member, _ = rows.pop(index)
                        if replacement is not None:
                            rows.append((member, replacement))
                        index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith(plugins.RECEIPT))
                        member, data = rows[index]
                        receipt = json.loads(data)
                        receipt['files'].pop(name)
                        if replacement is not None:
                            receipt['files'][name] = {'size_bytes': len(replacement), 'sha256': plugins.digest(replacement)}
                        rows[index] = member, plugins.json_bytes(receipt)
                    self.rewrite_archive(archive, mutate)
                    with self.assertRaisesRegex(ValueError, 'native projection'):
                        plugins.verify_archive(archive, VERSION, COMMIT)

    def test_invalid_workflow_names_and_descriptions_refuse_generation(self):
        self.add_workflows()
        template = {'workflow/references/codex-workflow-wrapper.md': self.content['workflow/references/codex-workflow-wrapper.md']}
        for slug in ['workflow', 'Bad', '-bad', 'bad--name', 'x' * 57, 'bad\\name']:
            with self.subTest(slug=slug), self.assertRaises(ValueError):
                plugins.workflow_wrapper_skills(dict(template, **{f'workflow/workflows/{slug}.md': b'---\ndescription: Task\n---\n'}))
        for source in [b'no metadata', b'---\ndescription: \n---\n', b'---\ndescription: >\n---\n',
                       b'---\ndescription: Task\nname: injected\n---\n']:
            with self.subTest(source=source), self.assertRaises(ValueError):
                plugins.workflow_wrapper_skills(dict(template, **{'workflow/workflows/paper-read.md': source}))

    def test_source_missing_modified_unlisted_or_linked_fails(self):
        path = self.source / 'workflow/SKILL.md'
        original = path.read_bytes()
        path.write_bytes(b'changed')
        with self.assertRaises(ValueError): self.build()
        path.unlink()
        with self.assertRaises(ValueError): self.build()
        path.write_bytes(original)
        extra = self.source / 'unlisted'
        extra.write_bytes(b'extra')
        with self.assertRaises(ValueError): self.build()
        extra.unlink()
        extra.symlink_to(path)
        with self.assertRaises(ValueError): self.build()
        extra.unlink()
        self.metadata['entries'].append(dict(self.metadata['entries'][0]))
        self.write_metadata()
        with self.assertRaises(ValueError): self.build()

    def test_existing_output_and_projection_collision_fail(self):
        self.build()
        with self.assertRaises(ValueError): self.build()
        self.content['SKILL.md'] = b'collision'
        with self.assertRaises(ValueError): plugins.project(self.content, 'codex', VERSION)
        self.metadata['entries'][0]['path'] = '../outside'
        self.write_metadata()
        with self.assertRaises(ValueError): self.build('bad')

    def rewrite_archive(self, archive, change):
        with tarfile.open(archive) as old:
            members = [(m, old.extractfile(m).read()) for m in old]
        change(members)
        with tarfile.open(archive, 'w:gz') as out:
            for member, data in members:
                member.size = len(data)
                out.addfile(member, io.BytesIO(data))

    def test_archive_changed_bytes_links_duplicates_and_foreign_source_fail(self):
        archive = self.build()[0]
        original = archive.read_bytes()
        for mutate in [
            lambda rows: rows.__setitem__(-1, (rows[-1][0], b'changed')),
            lambda rows: rows.append(rows[-1]),
            lambda rows: setattr(rows[-1][0], 'type', tarfile.SYMTYPE),
            lambda rows: (rows[-1][0].pax_headers.clear(), setattr(rows[-1][0], 'name', '../outside')),
        ]:
            archive.write_bytes(original)
            self.rewrite_archive(archive, mutate)
            with self.assertRaises(ValueError): plugins.verify_archive(archive, VERSION, COMMIT)
        archive.write_bytes(original)
        with self.assertRaises(ValueError): plugins.verify_archive(archive, VERSION, 'b' * 40)

    def test_self_consistent_runtime_change_is_rejected(self):
        archive = self.build()[0]
        def mutate(rows):
            bridge_index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith('/.mcp.json'))
            member, data = rows[bridge_index]
            changed = data.replace(b'"lite"', b'"full"')
            rows[bridge_index] = member, changed
            receipt_index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith(plugins.RECEIPT))
            member, data = rows[receipt_index]
            receipt = json.loads(data)
            receipt['files']['.mcp.json'] = {'size_bytes': len(changed), 'sha256': plugins.digest(changed)}
            rows[receipt_index] = member, plugins.json_bytes(receipt)
        self.rewrite_archive(archive, mutate)
        with self.assertRaises(ValueError): plugins.verify_archive(archive, VERSION, COMMIT)

    def test_coordinated_resource_and_receipt_change_cannot_keep_native_pack_hash(self):
        archive = self.build()[0]
        def mutate(rows):
            name = 'workflow/SKILL.md'
            target = plugins.SKILL_ROOT + 'SKILL.md'
            index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith('/' + target))
            member, data = rows[index]
            changed = data.replace(b'Use supplied evidence', b'Invent evidence')
            rows[index] = member, changed
            receipt_index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith(plugins.RECEIPT))
            member, data = rows[receipt_index]
            receipt = json.loads(data)
            fields = {'size_bytes': len(changed), 'sha256': plugins.digest(changed)}
            receipt['files'][target] = fields
            next(e for e in receipt['source']['entries'] if e['path'] == name).update(fields)
            # Even an updated embedded manifest must reproduce the externally
            # checked native pack hash, rather than merely echoing that hash.
            manifest = json.loads(receipt['source']['pack_manifest_json'])
            offset = 0
            for entry in manifest['entries']:
                if entry['path'] == name:
                    entry.update(fields)
                entry['payload_offset'] = offset
                offset += entry['size_bytes']
            receipt['source']['pack_manifest_json'] = json.dumps(manifest, separators=(',', ':'), sort_keys=True)
            rows[receipt_index] = member, plugins.json_bytes(receipt)
        self.rewrite_archive(archive, mutate)
        with self.assertRaisesRegex(ValueError, 'native pack digest'):
            plugins.verify_archive(archive, VERSION, COMMIT)

    def test_missing_payload_or_wrong_native_manifest_is_rejected(self):
        original = self.metadata['pack_manifest_json']
        for update in [dict(source_commit='e' * 40), dict(content_version='2.0.0-alpha.7'),
                       dict(content_root_sha256='e' * 64), dict(entries=[])]:
            manifest = json.loads(original)
            manifest.update(update)
            self.metadata['pack_manifest_json'] = json.dumps(manifest)
            self.write_metadata()
            with self.subTest(update=update), self.assertRaises(ValueError): self.build()

    def test_native_platforms_modes_and_wrong_binary(self):
        for target, data in BINARIES.items():
            self.binary.write_bytes(data)
            archive = self.build(target, target)[0]
            info = plugins.verify_archive(archive, VERSION, COMMIT)
            self.assertEqual(info['target'], target)
            with tarfile.open(archive) as packet:
                executable = next(m for m in packet if m.name.endswith('/' + plugins.binary_path(target)))
                self.assertEqual(executable.mode, 0o755)
            self.rewrite_archive(archive, lambda rows: setattr(
                next(m for m, _ in rows if m.name.endswith('/' + plugins.binary_path(target))), 'mode', 0o644))
            with self.assertRaisesRegex(ValueError, 'permissions'):
                plugins.verify_archive(archive, VERSION, COMMIT)
        with self.assertRaisesRegex(ValueError, 'architecture'):
            self.build('wrong-binary', TARGET)
        self.binary.unlink()
        with self.assertRaisesRegex(ValueError, 'regular file'):
            self.build('missing-binary')

    def test_legacy_npm_bridge_archive_still_verifies(self):
        files = plugins.project(self.content, 'codex', VERSION)
        source_names = ('.codex-plugin/plugin.json', '.claude-plugin/plugin.json')
        files[plugins.RECEIPT] = plugins.json_bytes({
            'schema_version': 1, 'platform': 'codex', 'source': self.metadata,
            'source_manifest_bytes': {n: self.content[n].decode() for n in source_names},
            'files': {n: {'size_bytes': len(d), 'sha256': plugins.digest(d)} for n, d in files.items()},
        })
        archive = self.root / plugins.archive_name('codex', VERSION)
        prefix = archive.name.removesuffix('.tar.gz') + '/plugins/qiongli-next/'
        with tarfile.open(archive, 'w:gz') as packet:
            for name, data in files.items():
                member = tarfile.TarInfo(prefix + name)
                member.size = len(data)
                packet.addfile(member, io.BytesIO(data))
        self.assertIsNone(plugins.verify_archive(archive, VERSION, COMMIT)['target'])

    def test_complete_platform_packet_binds_plugins_binaries_index_and_smoke(self):
        import shutil
        from tooling.scripts.native_cli_release import archive_cli
        from tooling.scripts.native_registry_packages import wheel
        from tooling.scripts.native_release_assets import assemble, verify
        tags = ['macosx_11_0_arm64', 'manylinux_2_35_x86_64', 'win_amd64', 'manylinux_2_35_aarch64']
        for (target, data), tag in zip(BINARIES.items(), tags):
            self.binary.write_bytes(data)
            folder = self.root / 'targets' / target
            folder.mkdir(parents=True)
            extension = 'zip' if target.endswith('msvc') else 'tar.gz'
            archive = folder / f'qiongli-{VERSION}-{target}.{extension}'
            archive_cli(archive, self.binary, b'fixture', target)
            whl = wheel(folder, plugins.parse_release_version(VERSION).package_version, tag, data, 'fixture')
            archives = ([self.legacy_archive(folder, host, target) for host in plugins.PLATFORMS]
                        if VERSION == '2.0.1' else self.build('plugins-' + target, target))
            checks = {}
            for host, path in zip(plugins.PLATFORMS, archives):
                if path.parent != folder:
                    shutil.copyfile(path, folder / path.name)
                checks[host] = dict(plugins.verify_archive(path, VERSION, COMMIT),
                                    status='passed', runtime_path='empty', mcp_tools=14)
            receipt = {'version': VERSION, 'source_commit': COMMIT, 'target': target,
                       'checks': {'cli_mcp_tests': 'passed', 'cli_clippy': 'passed',
                                  'npm_wheel_local_install': 'passed',
                                  'archive_smoke': {'version': VERSION, 'content_pack_sha256': self.metadata['pack_sha256'],
                                                    'runtime_path': 'empty', 'windows_system_dlls': ['KERNEL32.dll']},
                                  'marketplace_plugins': checks},
                       'artifacts': [{'file': p.name, 'sha256': plugins.digest(p.read_bytes()),
                                      'bytes': p.stat().st_size} for p in folder.iterdir()]}
            if VERSION == '2.0.1':
                golden = json.loads((Path(__file__).resolve().parents[1] / 'packages/qiongli-native/apps/qiongli/tests/fixtures/plugin-source-v1.status.json').read_text())
                golden.pop('destination')
                transition = {'codex': golden, 'claude': dict(golden, target='claude-code')}
                smoke = receipt['checks']['archive_smoke']
                smoke['plugin_source_transition'] = transition
                receipt['checks']['registry_install'] = {name: dict(smoke) for name in ('npm', 'pypi')}
                for host in checks.values():
                    host['plugin_source_transition'] = transition
            (folder / 'release-manifest.json').write_text(json.dumps(receipt))
        assets = self.root / 'combined/assets'
        assemble(self.root / 'targets', assets, VERSION, COMMIT)
        packet, _, _ = verify(assets, VERSION, COMMIT)
        self.assertEqual(len(packet['artifacts']), 18)
        index = json.loads((assets / 'marketplace-plugins.json').read_text())
        self.assertEqual(len(index['plugins']), 8)
        self.assertEqual({p['name'] for p in index['plugins']}, {plugins.plugin_name(t, VERSION, legacy_identity=VERSION == '2.0.1') for t in BINARIES})
        manifest = assets / 'release-manifest.json'
        if VERSION == '2.0.1':
            for location in (('archive_smoke',), ('registry_install', 'npm'),
                             ('registry_install', 'pypi'), ('marketplace_plugins', 'codex'),
                             ('marketplace_plugins', 'claude')):
                for wrong in (None, {'codex': {'schema_version': 2}},
                              {'codex': {'plugin_id': 'qiongli@qiongli-cli-local'}}):
                    changed = json.loads(json.dumps(packet))
                    evidence = changed['target_evidence'][0]['checks']
                    for key in location:
                        evidence = evidence[key]
                    evidence['plugin_source_transition'] = wrong
                    manifest.write_text(json.dumps(changed))
                    with self.assertRaisesRegex(ValueError, 'transition evidence'):
                        verify(assets, VERSION, COMMIT)
            manifest.write_text(json.dumps(packet))
            # Simulate a forward-ported helper accepting stable archive names.
            renamed = json.loads(json.dumps(packet))
            for artifact in renamed['artifacts']:
                if '-plugin-' in artifact['file']:
                    name = artifact['file'].replace('qiongli-next-', 'qiongli-', 1)
                    (assets / name).write_bytes((assets / artifact['file']).read_bytes())
                    artifact['file'] = name
            manifest.write_text(json.dumps(renamed))
            with patch('tooling.scripts.native_release_assets.archive_name',
                       side_effect=lambda *args: plugins.archive_name(*args).replace('qiongli-next-', 'qiongli-', 1)):
                with self.assertRaisesRegex(ValueError, 'Next marketplace'):
                    verify(assets, VERSION, COMMIT)
            manifest.write_text(json.dumps(packet))
            index_path = assets / 'marketplace-plugins.json'
            original_index = index_path.read_bytes()
            wrong_index = json.loads(original_index)
            wrong_index['plugins'][0].update(name='qiongli-macos-arm64', plugin_path='plugins/qiongli-macos-arm64')
            index_path.write_text(json.dumps(wrong_index))
            changed = json.loads(json.dumps(packet))
            entry = next(a for a in changed['artifacts'] if a['file'] == index_path.name)
            entry.update(sha256=plugins.digest(index_path.read_bytes()), bytes=index_path.stat().st_size)
            manifest.write_text(json.dumps(changed))
            with patch('tooling.scripts.native_release_assets.marketplace_index', return_value=wrong_index):
                with self.assertRaisesRegex(ValueError, 'legacy Next marketplace identities'):
                    verify(assets, VERSION, COMMIT)
            index_path.write_bytes(original_index)
            manifest.write_text(json.dumps(packet))
        cli_archive = assets / f'qiongli-{VERSION}-{TARGET}.tar.gz'
        original = cli_archive.read_bytes()
        self.rewrite_archive(cli_archive, lambda rows: rows.__setitem__(0, (rows[0][0], BINARIES[TARGET] + b'changed')))
        changed_packet = json.loads(json.dumps(packet))
        entry = next(p for p in changed_packet['artifacts'] if p['file'] == cli_archive.name)
        entry.update(sha256=plugins.digest(cli_archive.read_bytes()), bytes=cli_archive.stat().st_size)
        manifest.write_text(json.dumps(changed_packet))
        with self.assertRaisesRegex(ValueError, 'CLI executable differs'):
            verify(assets, VERSION, COMMIT)
        cli_archive.write_bytes(original)
        for change, error in [
            (lambda p: p['artifacts'].remove(next(a for a in p['artifacts'] if a['file'] == 'marketplace-plugins.json')), 'platform index mismatch'),
            (lambda p: p['target_evidence'][0]['checks']['archive_smoke'].update(runtime_path='inherited'), 'empty-PATH CLI'),
            (lambda p: next(r for r in p['target_evidence'] if r['target'].endswith('msvc'))['checks']['archive_smoke'].pop('windows_system_dlls'), 'system-DLL'),
            (lambda p: p['target_evidence'][0]['checks']['marketplace_plugins']['codex'].update(runtime_path='inherited'), 'smoke evidence'),
            (lambda p: p['artifacts'].remove(next(a for a in p['artifacts'] if '-codex-plugin-' in a['file'])), 'all target-specific'),
            (lambda p: p.update(artifacts=[a for a in p['artifacts'] if '-plugin-' not in a['file']]), 'all target-specific'),
        ]:
            modified = json.loads(json.dumps(packet))
            change(modified)
            manifest.write_text(json.dumps(modified))
            with self.assertRaisesRegex(ValueError, error): verify(assets, VERSION, COMMIT)
        manifest.write_text(json.dumps(packet))
        target_index = assets / 'marketplace-plugins.json'
        index['plugins'][0]['plugin_path'] = 'plugins/wrong-platform'
        target_index.write_text(json.dumps(index))
        entry = next(p for p in packet['artifacts'] if p['file'] == target_index.name)
        entry.update(sha256=plugins.digest(target_index.read_bytes()), bytes=target_index.stat().st_size)
        manifest.write_text(json.dumps(packet))
        with self.assertRaisesRegex(ValueError, 'legacy Next marketplace identities' if VERSION == '2.0.1' else 'platform index mismatch'):
            verify(assets, VERSION, COMMIT)
        # Even self-consistent archive hashes cannot substitute another executable.
        packet = json.loads(manifest.read_text())
        changed_archive = assets / plugins.archive_name('codex', VERSION, TARGET, legacy_identity=VERSION == '2.0.1')
        def mutate(rows):
            path = plugins.binary_path(TARGET)
            binary_index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith('/' + path))
            member, data = rows[binary_index]
            changed = data + b'foreign executable'
            rows[binary_index] = member, changed
            receipt_index = next(i for i, (m, _) in enumerate(rows) if m.name.endswith(plugins.RECEIPT))
            member, data = rows[receipt_index]
            receipt = json.loads(data)
            receipt['files'][path] = {'size_bytes': len(changed), 'sha256': plugins.digest(changed)}
            rows[receipt_index] = member, plugins.json_bytes(receipt)
        self.rewrite_archive(changed_archive, mutate)
        entry = next(p for p in packet['artifacts'] if p['file'] == changed_archive.name)
        entry.update(sha256=plugins.digest(changed_archive.read_bytes()), bytes=changed_archive.stat().st_size)
        manifest.write_text(json.dumps(packet))
        with self.assertRaisesRegex(ValueError, 'executable differs'):
            verify(assets, VERSION, COMMIT)

    def test_201_packet_requires_transition_evidence_and_retains_next_identity(self):
        with patch(__name__ + '.VERSION', '2.0.1'):
            for host in plugins.PLATFORMS:
                name = f'.{host}-plugin/plugin.json'
                self.content[name] = plugins.json_bytes({'name': 'qiongli', 'version': VERSION, 'skills': './'})
            self.write_source()
            self.test_complete_platform_packet_binds_plugins_binaries_index_and_smoke()

    def test_bridge_windows_shell_has_only_fixed_tokens_and_preserves_exit(self):
        harness = r'''
import vm from 'node:vm';
const source = JSON.parse(process.argv[1]).replace("import { spawn } from 'node:child_process';", '');
const rows = [];
for (const platform of ['linux', 'win32']) {
  const events = {};
  const fakeProcess = {platform, argv: ['node', 'bridge', '; injected'], on() {}, exitCode: 0};
  const spawn = (...args) => {rows.push(args); return {on(name, fn) {events[name] = fn;}, kill() {}};};
  vm.runInNewContext(source, {process: fakeProcess, spawn, console});
  events.close(7, null);
  if (fakeProcess.exitCode !== 7) throw new Error('exit code lost');
}
console.log(JSON.stringify(rows));
'''
        result = subprocess.run(['node', '--input-type=module', '--eval', harness,
                                 json.dumps(plugins.bridge(VERSION).decode())], check=True, capture_output=True, text=True)
        linux, windows = json.loads(result.stdout)
        args = ['--yes', f'qiongli@{VERSION}', 'mcp', 'serve', '--profile', 'lite', '--transport', 'stdio']
        self.assertEqual(linux, ['npx', args, {'stdio': 'inherit', 'shell': False}])
        self.assertEqual(windows, ['npx.cmd ' + ' '.join(args), [], {'stdio': 'inherit', 'shell': True}])
        self.assertNotIn('injected', result.stdout)


if __name__ == '__main__':
    unittest.main()
