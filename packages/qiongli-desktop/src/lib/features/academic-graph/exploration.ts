import type { AcademicGraphNode } from '@qiongli/app-api';

const MAX_FOCUS_HISTORY = 50;

export interface AcademicGraphSearchMatch {
  nodeId: string;
  label: string;
  canonicalId: string;
  score: number;
}

export interface AcademicGraphFocusHistory {
  entries: string[];
  index: number;
}

export function findAcademicGraphMatches(
  nodes: AcademicGraphNode[],
  query: string,
  limit = 8
): AcademicGraphSearchMatch[] {
  const normalized = query.trim().toLocaleLowerCase();
  if (normalized.length === 0 || limit <= 0) return [];
  return nodes
    .map((node) => {
      const label = node.label.toLocaleLowerCase();
      const canonicalId = node.canonicalId.toLocaleLowerCase();
      let score = Number.POSITIVE_INFINITY;
      if (label === normalized || canonicalId === normalized) score = 0;
      else if (canonicalId.startsWith(normalized)) score = 1;
      else if (label.startsWith(normalized)) score = 2;
      else if (canonicalId.includes(normalized)) score = 3;
      else if (label.includes(normalized)) score = 4;
      return {
        nodeId: node.nodeId,
        label: node.label,
        canonicalId: node.canonicalId,
        score
      };
    })
    .filter((match) => Number.isFinite(match.score))
    .sort((left, right) =>
      left.score - right.score
      || left.canonicalId.localeCompare(right.canonicalId)
      || left.label.localeCompare(right.label)
      || left.nodeId.localeCompare(right.nodeId))
    .slice(0, limit);
}

export function pushAcademicGraphFocus(
  history: AcademicGraphFocusHistory,
  nodeId: string
): AcademicGraphFocusHistory {
  if (history.entries[history.index] === nodeId) return history;
  const entries = [...history.entries.slice(0, history.index + 1), nodeId]
    .slice(-MAX_FOCUS_HISTORY);
  return { entries, index: entries.length - 1 };
}

export function moveAcademicGraphFocus(
  history: AcademicGraphFocusHistory,
  offset: -1 | 1
): AcademicGraphFocusHistory {
  const index = Math.max(0, Math.min(history.entries.length - 1, history.index + offset));
  return { entries: history.entries, index };
}
