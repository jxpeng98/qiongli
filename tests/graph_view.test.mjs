import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import test from 'node:test';
import { setImmediate as nextTurn } from 'node:timers/promises';

// Reuse the repository's existing development dependency; the exported HTML ships none.
const require = createRequire(new URL('../packages/qiongli-desktop/package.json', import.meta.url));
const { JSDOM, VirtualConsole } = require('jsdom');
const template = readFileSync(new URL('../packages/qiongli-native/apps/qiongli/src/graph_view.html', import.meta.url), 'utf8');
const id = (kind, number) => kind + '_' + number.toString(16).padStart(64, '0');
function fixture() {
  const claim = { nodeId: id('nod', 0), canonicalId: 'C-1', nodeType: 'claim', label: 'Exposure and returns', artifactPath: 'analysis/evidence.csv', sourceAnchor: 'claim:C-1' };
  const records = Array.from({ length: 104 }, (_, i) => ({
    nodeId: id('nod', i + 1), canonicalId: 'E-' + (i + 1),
    nodeType: i < 16 ? 'evidence' : 'paper',
    label: i === 0 ? '证据 😀 <img src=x onerror=alert(1)>' : 'Research record ' + (i + 1),
    artifactPath: i < 16 ? 'analysis/evidence.csv' : 'literature/paper-' + i + '.md',
    sourceAnchor: 'evidence:' + (i + 1)
  }));
  const edges = Array.from({ length: 20 }, (_, i) => ({
    edgeId: id('edg', i), sourceNodeId: records[i % 16].nodeId, targetNodeId: claim.nodeId,
    relation: i < 17 ? 'supports' : ['contradicts', 'weakens', 'cites'][i - 17],
    status: i < 17 ? 'reviewed' : ['proposed', 'rejected', 'observed'][i - 17],
    rationale: 'Recorded finding ' + i, evidenceLimit: 'One population; no causal identification.',
    artifactPath: 'analysis/evidence.csv', sourceAnchor: 'support:' + i
  }));
  edges[16].sourceNodeId = claim.nodeId; // Self-relation must not create NaN geometry.
  return {
    snapshot: { projectId: 'prj_test', projectRevision: 7, projectionId: id('grp', 1),
      nodes: [claim, ...records], edges, diagnostics: [], sourceCount: 2, presentSourceCount: 1 },
    readiness: { state: 'sparse', sources: [{ artifactPath: 'analysis/evidence.csv', state: 'present', freshness: 'stale' }] }
  };
}
function page(data) {
  const errors = [], console = new VirtualConsole();
  console.on('jsdomError', error => errors.push(error.message));
  const html = template.replace('__QIONGLI_GRAPH_DATA__', JSON.stringify(data).replaceAll('<', '\\u003c'));
  const dom = new JSDOM(html, { runScripts: 'dangerously', virtualConsole: console });
  const { document } = dom.window;
  return {
    dom, document, errors, $: id => document.getElementById(id),
    change(id, value) {
      const input = document.getElementById(id); input.value = value;
      input.dispatchEvent(new dom.window.Event(id === 'search' ? 'input' : 'change'));
    }
  };
}

test('worked example shows actual CLI evidence, proposed decisions and the unsupported claim', () => {
  const data = JSON.parse(readFileSync(new URL('../docs/public/demos/research-graph.snapshot.json', import.meta.url), 'utf8'));
  const source = JSON.parse(readFileSync(new URL('../docs/public/demos/research-graph.source.json', import.meta.url), 'utf8')).artifact;
  const { dom, document, errors, $, change } = page(data);
  const choose = id => [...document.querySelectorAll('#nodes button')].find(button => button.querySelector('small').textContent.endsWith(' · ' + id)).click();
  try {
    assert.equal(document.querySelectorAll('#nodes button').length, 9);
    choose('CLM-1');
    const support = document.querySelector('#relations [data-edge-id="' + source.entityId + '"]');
    assert.ok(support);
    support.click();
    assert.match($('fields').textContent, /no random assignment; not causal/);
    assert.ok($('source').textContent.includes(source.projectionId));
    assert.ok($('source').textContent.endsWith(source.entityId));
    choose('CLM-2');
    assert.equal(document.querySelectorAll('#relations button').length, 1);
    assert.match($('relations').textContent, /informs/);
    assert.match($('relations').textContent, /proposed/);
    assert.ok(!$('relations').textContent.includes('supports'));
    assert.match($('diagnostics').textContent, /CLM-2/);
    change('relation-status', 'reviewed');
    assert.match($('scope').textContent, /No relations match/);
    change('relation-status', 'proposed');
    assert.equal(document.querySelectorAll('#relations button').length, 1);
    assert.deepEqual(errors, []);
  } finally { dom.window.close(); }
});

test('offline graph explores every page, preserves direction/status, and retains exact source bindings', async () => {
  const data = fixture(), original = JSON.stringify(data);
  const { dom, document, errors, $, change } = page(data);
  try {
    assert.equal(document.querySelectorAll('#nodes button').length, 100);
    $('records-next').click();
    assert.equal(document.activeElement, document.querySelector('#nodes button'));
    assert.equal(document.querySelectorAll('#nodes button').length, 5);
    assert.equal($('records-next').disabled, true);
    change('node-type', 'evidence');
    assert.equal(document.querySelectorAll('#nodes button').length, 16);
    assert.equal($('records-prev').disabled, true);
    assert.match($('count').textContent, /outside these filters/);
    change('search', '证据');
    assert.equal(document.querySelectorAll('#nodes button').length, 1);
    assert.equal(document.querySelector('img'), null);
    change('search', 'absent');
    assert.match($('count').textContent, /No matching records/);
    change('node-type', ''); change('search', '');

    const seen = new Set();
    do {
      const paths = [...document.querySelectorAll('#map .edge')];
      assert.ok(paths.length > 0 && paths.length <= 8);
      assert.ok(document.querySelectorAll('#map g[role=button]').length <= 9);
      for (const path of paths) {
        seen.add(path.dataset.edgeId);
        assert.equal(path.getAttribute('marker-end'), 'url(#arrow)');
        assert.ok(!/NaN|Infinity/.test(path.getAttribute('d')));
      }
      if ($('relations-next').disabled) break;
      $('relations-next').click();
      assert.equal(document.activeElement, document.querySelector('#relations button'));
    } while (true);
    assert.equal(seen.size, 20, 'no relation disappears at the display limit');
    assert.match($('scope').textContent, /17–20 of 20/);
    change('relation-type', 'contradicts'); change('relation-status', 'proposed');
    const path = document.querySelector('#map .edge');
    assert.equal(path.dataset.status, 'proposed');
    assert.equal(path.dataset.relation, 'contradicts');
    assert.match(path.getAttribute('aria-label'), /Research record 2 → contradicts → Exposure and returns/);
    path.dispatchEvent(new dom.window.KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    assert.equal(document.activeElement, $('detail-title'));
    assert.equal(path.getAttribute('aria-pressed'), 'true');
    assert.match($('fields').textContent, /no causal identification/);
    assert.match($('source').textContent, new RegExp('--edge-id ' + id('edg', 17)));
    assert.match($('source').textContent, /--expected-project-revision 7/);
    assert.ok(!$('source').textContent.includes('analysis/'));

    let copied;
    Object.defineProperty(dom.window.navigator, 'clipboard', { configurable: true, value: { writeText: async text => { copied = text; } } });
    $('copy').click(); await nextTurn();
    assert.equal(copied, $('source').textContent);
    assert.match($('copy-status').textContent, /Command copied/);
    Object.defineProperty(dom.window.navigator, 'clipboard', { configurable: true, value: { writeText: async () => { throw new Error('denied'); } } });
    $('copy').click(); await nextTurn();
    assert.equal(dom.window.getSelection().toString(), $('source').textContent);
    assert.match($('copy-status').textContent, /copy it with your keyboard/);

    let denyOldCopy;
    Object.defineProperty(dom.window.navigator, 'clipboard', { configurable: true, value: {
      writeText: () => new Promise((resolve, reject) => { denyOldCopy = reject; })
    } });
    $('copy').click();
    const neighbor = [...document.querySelectorAll('#map g[role=button]')].find(n => n.getAttribute('aria-pressed') === 'false');
    neighbor.dispatchEvent(new dom.window.KeyboardEvent('keydown', { key: ' ', bubbles: true }));
    assert.notEqual($('selected').textContent, 'Exposure and returns');
    denyOldCopy(new Error('late permission denial')); await nextTurn();
    assert.equal($('copy-status').textContent, '', 'a delayed copy result must not describe a different record');
    assert.equal($('back').disabled, false);
    $('back').click();
    assert.equal($('selected').textContent, 'Exposure and returns');
    assert.equal($('back').disabled, true);
    change('relation-status', 'reviewed');
    assert.match($('scope').textContent, /No relations match/);
    assert.equal(document.querySelectorAll('#map .edge').length, 0);
    assert.equal($('relations-next').disabled, true);

    document.querySelector('#diagnostics button').click();
    assert.equal(document.activeElement, $('search'));
    assert.equal($('search').value, 'analysis/evidence.csv');
    assert.equal(document.querySelectorAll('#nodes button').length, 17);
    assert.ok(!$('diagnostics').textContent.includes('No source issues'));

    let saved, filename;
    dom.window.URL.createObjectURL = blob => { saved = blob; return 'blob:synthetic'; };
    dom.window.URL.revokeObjectURL = () => {};
    dom.window.HTMLAnchorElement.prototype.click = function () { filename = this.download; };
    $('download').click();
    const reader = new dom.window.FileReader();
    const loaded = new Promise(resolve => { reader.onload = () => resolve(reader.result); });
    reader.readAsText(saved);
    assert.deepEqual(JSON.parse(await loaded), data);
    assert.equal(filename, 'research-graph-r7.json');
    assert.equal(JSON.stringify(data), original);
    assert.deepEqual(errors, []);
  } finally { dom.window.close(); }
});

test('empty, isolated and invalid-identity snapshots do not invent source health or executable commands', () => {
  const empty = fixture();
  empty.snapshot.nodes = []; empty.snapshot.edges = [];
  empty.readiness.sources[0].state = 'missing';
  let view = page(empty);
  try {
    assert.equal(view.$('detail').hidden, true);
    assert.equal(view.$('relations-next').disabled, true);
    assert.match(view.$('scope').textContent, /No research records/);
    assert.match(view.$('diagnostics').textContent, /missing/);
    assert.equal(view.document.querySelector('#diagnostics button'), null);
    assert.deepEqual(view.errors, []);
  } finally { view.dom.window.close(); }
  const isolated = fixture();
  isolated.snapshot.nodes = [isolated.snapshot.nodes[0]]; isolated.snapshot.edges = [];
  isolated.snapshot.nodes[0].nodeId = 'nod_unsafe; touch file';
  isolated.readiness.sources = [];
  view = page(isolated);
  try {
    assert.match(view.$('scope').textContent, /No semantic relations/);
    assert.match(view.$('source').textContent, /invalid snapshot identifiers/);
    assert.equal(view.$('copy').disabled, true);
    assert.match(view.$('diagnostics').textContent, /No source issues recorded/);
    assert.deepEqual(view.errors, []);
  } finally { view.dom.window.close(); }
});

test('light and dark theme text pairs meet contrast without mixing media-query palettes', () => {
  const roots = [...template.matchAll(/:root\{([^}]+)\}/g)].map(match =>
    Object.fromEntries([...match[1].matchAll(/--([\w-]+):(#\w+)/g)].map(m => [m[1], m[2]])));
  assert.equal(roots.length, 2);
  const luminance = hex => {
    const expanded = hex.length === 4 ? '#' + [...hex.slice(1)].map(c => c + c).join('') : hex;
    return expanded.slice(1).match(/../g).map(c => parseInt(c, 16) / 255)
      .map(c => c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4)
      .reduce((sum, value, i) => sum + value * [.2126, .7152, .0722][i], 0);
  };
  for (const theme of roots) for (const [foreground, background, minimum] of [
    ['ink','paper',4.5], ['muted','paper',4.5], ['ink','panel',4.5],
    ['muted','panel',4.5], ['ink','tint',4.5], ['panel','accent',4.5],
    ['accent','panel',3], ['challenge','panel',3], ['muted','panel',3]
  ]) {
    const values = [luminance(theme[foreground]), luminance(theme[background])].sort((a,b) => b-a);
    assert.ok((values[0]+.05)/(values[1]+.05) >= minimum, foreground + ' on ' + background);
  }
});
