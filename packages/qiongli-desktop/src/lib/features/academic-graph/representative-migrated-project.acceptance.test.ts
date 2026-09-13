// @ts-expect-error Vitest runs this acceptance in Node; the Desktop bundle
// intentionally does not depend on Node type declarations.
import { createHash } from 'node:crypto';
// @ts-expect-error See the Node-only acceptance note above.
import { existsSync, lstatSync, mkdtempSync, readFileSync, readdirSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
// @ts-expect-error See the Node-only acceptance note above.
import { join } from 'node:path';
// @ts-expect-error See the Node-only acceptance note above.
import { tmpdir } from 'node:os';
// @ts-expect-error See the Node-only acceptance note above.
import { spawnSync } from 'node:child_process';
// @ts-expect-error See the Node-only acceptance note above.
import { env as processEnvironment } from 'node:process';

import {
  academicGraphQueryResultSchema,
  academicGraphReadinessSchema,
  academicGraphSnapshotSchema,
  appEventSchema
} from '@qiongli/app-api';
import { describe, expect, it } from 'vitest';

import { buildCytoscapeElements } from './cytoscape-adapter';
import { findAcademicGraphMatches } from './exploration';
import {
  artifactForAcademicGraphEntity,
  buildAcademicGraphInspection
} from './inspection';
import { buildAcademicGraphLayout, buildAcademicGraphViewState } from './layout';
import { canRenderAcademicGraph } from './readiness';

type ProcessEnvironment = Record<string, string | undefined>;
type DirectoryEntry = {
  name: string;
  isDirectory(): boolean;
  isFile(): boolean;
};

const source = processEnvironment.PLT322_MIGRATION_SOURCE;
const native = processEnvironment.PLT322_NATIVE_BINARY;
const factsPath = processEnvironment.PLT322_FACTS_PATH;
const canonicalInventoryDigest = processEnvironment.PLT322_SOURCE_INVENTORY_DIGEST;
const enabled = Boolean(source && native && factsPath && canonicalInventoryDigest);

describe.runIf(enabled)('PLT-322 representative migrated project', () => {
  it('preserves scholarly semantics through migration, query, inspection, and presentation', () => {
    const executionRoot = mkdtempSync(join(realpathSync(tmpdir()), 'qiongli-plt322-'));
    const configHome = join(executionRoot, 'config');
    const destination = join(executionRoot, 'migrated');
    const env = { ...processEnvironment, QIONGLI_CONFIG_HOME: configHome };
    const beforeInventory = inventoryDigest(source!);
    try {
      expect(existsSync(join(source!, '.qiongli', 'session.json'))).toBe(true);
      expect(existsSync(join(source!, '.claude', 'transcripts', 'acceptance.txt'))).toBe(true);
      expect(existsSync(join(source!, 'context', 'project_manifest.json'))).toBe(false);

      const previewOutput = runJson([
        'project', 'migrate', 'preview',
        '--source', source!, '--root', destination,
        '--name', 'asset-pricing-capm-ff3', '--kind', 'article', '--stage', 'analysis'
      ], env);
      const preview = previewOutput.preview;
      expect(preview.schemaVersion).toBe(2);
      expect(preview.sourceRetained).toBe(true);
      expect(preview.excludedEntryCount).toBeGreaterThanOrEqual(2);
      expect(preview.copiedFileCount).toBeGreaterThan(0);

      const migrationOutput = runJson([
        'project', 'migrate', 'apply',
        '--source', source!, '--root', destination,
        '--name', 'asset-pricing-capm-ff3', '--kind', 'article', '--stage', 'analysis',
        '--project-id', preview.projectId,
        '--manifest-created-at-unix', String(preview.manifestCreatedAtUnix),
        '--expected-plan-digest', preview.planDigest,
        '--approve-filesystem-write'
      ], env);
      expect(migrationOutput.commit).toMatchObject({
        projectId: preview.projectId,
        sourceRetained: true,
        excludedEntryCount: preview.excludedEntryCount,
        indexRebuildRequired: true
      });
      expect(existsSync(join(destination, 'context', 'project_manifest.json'))).toBe(true);
      expect(existsSync(join(destination, '.claude'))).toBe(false);
      expect(existsSync(join(destination, '.qiongli', 'session.json'))).toBe(false);
      expect(inventoryDigest(source!)).toBe(beforeInventory);

      const first = snapshot(preview.projectId, env);
      const second = snapshot(preview.projectId, env);
      runJson(['project', 'show', '--project-id', preview.projectId], env);
      const reopened = snapshot(preview.projectId, env);
      expect(identity(first.graph)).toEqual(identity(second.graph));
      expect(identity(first.graph)).toEqual(identity(reopened.graph));
      expect(first.readiness.state).toBe('visualizable');
      expect(first.readiness.reasonCode).toBe('academic-graph-visualizable');
      expect(first.graph.diagnosticCount).toBe(0);

      const semanticNodes = first.graph.nodes.filter((node) =>
        node.nodeType !== 'project' && node.nodeType !== 'artifact');
      const reviewedRelations = first.graph.edges.filter((edge) =>
        edge.relation !== 'contains'
        && edge.status === 'reviewed'
        && edge.artifactPath !== 'graph/semantic_links.jsonl');
      expect(semanticNodes.length).toBeGreaterThan(0);
      expect(reviewedRelations.length).toBeGreaterThan(0);
      expect([...semanticNodes, ...reviewedRelations].every((entity) =>
        !entity.artifactPath.startsWith('/') && entity.sourceAnchor.length > 0)).toBe(true);

      const fullQuery = query(preview.projectId, first.graph.projectionId, [], env);
      const repeatedQuery = query(preview.projectId, reopened.graph.projectionId, [], env);
      expect(fullQuery).toEqual(repeatedQuery);
      expect(fullQuery.nodes.length).toBe(first.graph.nodeCount);
      expect(fullQuery.edges.length).toBe(first.graph.edgeCount);

      const stableIdQuery = query(preview.projectId, first.graph.projectionId, [
        '--canonical-id', 'CLM-001'
      ], env);
      expect(stableIdQuery.nodes.some((node) => node.canonicalId === 'CLM-001')).toBe(true);
      const relationQuery = query(preview.projectId, first.graph.projectionId, [
        '--relation', 'supports'
      ], env);
      expect(relationQuery.edges.length).toBeGreaterThan(0);
      expect(relationQuery.edges.every((edge) => edge.relation === 'supports')).toBe(true);

      const doctor = runJson([
        'project', 'graph', 'doctor', '--project-id', preview.projectId
      ], env);
      expect(doctor).toMatchObject({
        deterministicRebuild: true,
        projectionId: first.graph.projectionId,
        nodeCount: first.graph.nodeCount,
        edgeCount: first.graph.edgeCount
      });

      const selectedNode = stableIdQuery.nodes.find((node) => node.canonicalId === 'CLM-001')!;
      const selectedEdge = relationQuery.edges[0]!;
      const nodeArtifact = readArtifact(
        preview.projectId,
        first.graph.projectRevision,
        first.graph.projectionId,
        'node',
        selectedNode.nodeId,
        env
      );
      const edgeArtifact = readArtifact(
        preview.projectId,
        first.graph.projectRevision,
        first.graph.projectionId,
        'edge',
        selectedEdge.edgeId,
        env
      );
      expect(nodeArtifact.artifactPath).toBe(selectedNode.artifactPath);
      expect(nodeArtifact.sourceAnchor).toBe(selectedNode.sourceAnchor);
      expect(edgeArtifact.artifactPath).toBe(selectedEdge.artifactPath);
      expect(edgeArtifact.sourceAnchor).toBe(selectedEdge.sourceAnchor);

      const layout = buildAcademicGraphLayout(fullQuery, undefined, { performanceBudgetMs: 10_000 });
      const elements = buildCytoscapeElements(layout);
      const matches = findAcademicGraphMatches(fullQuery.nodes, 'CLM-001');
      const view = buildAcademicGraphViewState(layout, {
        selectedNodeId: selectedNode.nodeId,
        selectedEdgeId: selectedEdge.edgeId,
        focusNodeId: selectedNode.nodeId,
        direction: 'both',
        matchingNodeIds: matches.map((match) => match.nodeId)
      });
      const nodeInspection = buildAcademicGraphInspection(first.graph, {
        kind: 'node', id: selectedNode.nodeId
      });
      const edgeInspection = buildAcademicGraphInspection(first.graph, {
        kind: 'edge', id: selectedEdge.edgeId
      });
      expect(canRenderAcademicGraph(first.readiness, fullQuery)).toBe(true);
      expect(layout.nodes.length).toBe(fullQuery.nodes.length);
      expect(layout.edges.length).toBe(fullQuery.edges.length);
      expect(elements.length).toBeGreaterThanOrEqual(layout.nodes.length + layout.edges.length);
      expect(matches[0]?.nodeId).toBe(selectedNode.nodeId);
      expect(view.focusNodeId).toBe(selectedNode.nodeId);
      expect(nodeInspection?.artifactPath).toBe(nodeArtifact.artifactPath);
      expect(edgeInspection?.artifactPath).toBe(edgeArtifact.artifactPath);
      expect(artifactForAcademicGraphEntity(
        nodeArtifact,
        first.graph,
        { kind: 'node', id: selectedNode.nodeId }
      )).toBe(nodeArtifact);
      expect(artifactForAcademicGraphEntity(
        edgeArtifact,
        first.graph,
        { kind: 'edge', id: selectedEdge.edgeId }
      )).toBe(edgeArtifact);

      const facts = {
        schemaVersion: 1,
        projectId: preview.projectId,
        migrationPlanDigest: preview.planDigest,
        copiedFileCount: migrationOutput.commit.copiedFileCount,
        excludedEntryCount: migrationOutput.commit.excludedEntryCount,
        sourceRetained: migrationOutput.commit.sourceRetained,
        sourceInventoryDigest: canonicalInventoryDigest,
        migrationInputDigest: beforeInventory,
        analysisResultsDigest: sha256(readFileSync(join(
          source!, 'analysis', 'results', 'output_digests.json'
        ))),
        projectionId: first.graph.projectionId,
        projectionDigest: first.graph.projectionDigest,
        graphSourceDigest: first.graph.graphSourceDigest,
        indexId: fullQuery.indexId,
        nodeCount: first.graph.nodeCount,
        semanticNodeCount: first.readiness.semanticNodeCount,
        edgeCount: first.graph.edgeCount,
        diagnosticCount: first.graph.diagnosticCount,
        readinessState: first.readiness.state,
        reasonCode: first.readiness.reasonCode,
        nodeTypes: first.readiness.nodeTypeCounts.map((item) => item.nodeType).sort(),
        relations: first.readiness.relationCounts.map((item) => item.relation).sort(),
        checks: [
          'migration-preview-apply',
          'private-state-exclusion',
          'source-inventory-retained',
          'deterministic-rebuild',
          'fresh-process-reopen',
          'canonical-semantic-authority',
          'stable-id-query',
          'relation-query',
          'app-node-artifact-read',
          'app-edge-artifact-read',
          'desktop-readiness-layout',
          'desktop-search-focus',
          'desktop-source-inspection'
        ]
      };
      writeFileSync(factsPath!, `${JSON.stringify(facts, null, 2)}\n`, { mode: 0o600 });
    } finally {
      rmSync(executionRoot, { recursive: true, force: true });
    }
  }, 120_000);
});

function runJson(args: string[], env: ProcessEnvironment): Record<string, any> {
  const outcome = spawnSync(native!, args, {
    env,
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    timeout: 60_000
  });
  if (outcome.error || outcome.status !== 0) {
    throw new Error(`native command failed: ${outcome.stderr || outcome.error?.message}`);
  }
  return JSON.parse(outcome.stdout);
}

function snapshot(projectId: string, env: ProcessEnvironment) {
  const output = runJson(['project', 'graph', 'snapshot', '--project-id', projectId], env);
  return {
    graph: academicGraphSnapshotSchema.parse(output.snapshot),
    readiness: academicGraphReadinessSchema.parse(output.readiness)
  };
}

function query(
  projectId: string,
  projectionId: string,
  filters: string[],
  env: ProcessEnvironment
) {
  const output = runJson([
    'project', 'graph', 'query',
    '--project-id', projectId,
    '--expected-projection-id', projectionId,
    ...filters
  ], env);
  return academicGraphQueryResultSchema.parse(output.result);
}

function readArtifact(
  projectId: string,
  revision: number,
  projectionId: string,
  kind: 'node' | 'edge',
  id: string,
  env: ProcessEnvironment
) {
  const event = appEventSchema.parse(runJson([
    'app', 'read-project-artifact',
    '--project-id', projectId,
    '--expected-project-revision', String(revision),
    '--expected-projection-id', projectionId,
    `--${kind}-id`, id
  ], env));
  if (event.type !== 'project-artifact-read') throw new Error('unexpected artifact event');
  return event.artifact;
}

function identity(graph: ReturnType<typeof academicGraphSnapshotSchema.parse>) {
  return {
    projectionId: graph.projectionId,
    projectionDigest: graph.projectionDigest,
    graphSourceDigest: graph.graphSourceDigest,
    projectSemanticDigest: graph.projectSemanticDigest,
    nodes: graph.nodes.map((node) => node.nodeId),
    edges: graph.edges.map((edge) => edge.edgeId),
    diagnostics: graph.diagnostics
  };
}

function inventoryDigest(root: string): string {
  const hash = createHash('sha256');
  const visit = (directory: string, prefix = '') => {
    for (const entry of (readdirSync(directory, { withFileTypes: true }) as DirectoryEntry[])
      .sort((left: DirectoryEntry, right: DirectoryEntry) =>
        left.name < right.name ? -1 : left.name > right.name ? 1 : 0)) {
      const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
      const path = join(directory, entry.name);
      if (entry.isDirectory()) visit(path, relative);
      else {
        const metadata = lstatSync(path);
        if (!entry.isFile() || metadata.isSymbolicLink()) {
          throw new Error('migration source contains a non-regular entry');
        }
        const bytes = readFileSync(path);
        hash.update(relative).update('\0').update(String(bytes.length)).update('\0').update(bytes);
      }
    }
  };
  visit(root);
  return hash.digest('hex');
}

function sha256(bytes: Uint8Array): string {
  return createHash('sha256').update(bytes).digest('hex');
}
