import type {
  AcademicGraphQueryResult,
  AcademicGraphReadiness,
  AcademicGraphReadinessState,
  StatusCode
} from '@qiongli/app-api';

export function effectiveAcademicGraphReadiness(
  readiness: AcademicGraphReadiness,
  result: AcademicGraphQueryResult
): AcademicGraphReadinessState {
  return result.nodesTruncated || result.edgesTruncated
    ? 'bounded-truncated'
    : readiness.state;
}

export function academicGraphReadinessStatus(
  state: AcademicGraphReadinessState
): StatusCode {
  if (state === 'visualizable') return 'ready';
  if (state === 'empty-project' || state === 'no-recognized-artifacts') return 'missing';
  return 'attention';
}

export function canRenderAcademicGraph(
  readiness: AcademicGraphReadiness,
  result: AcademicGraphQueryResult
): boolean {
  return readiness.state !== 'empty-project'
    && readiness.state !== 'no-recognized-artifacts'
    && result.nodes.length > 0;
}
