// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { SvelteMap } from "svelte/reactivity";
import type { AccessPairResult } from "@kerolox/proto-ts";

export interface AccessWindowLite {
  startMs: number;
  endMs: number;
}

export interface AoiStats {
  count: number;
  totalAccessSeconds: number;
  meanGapSeconds: number | null;
  medianGapSeconds: number | null;
  maxGapSeconds: number | null;
}

export interface AoiAccessState {
  windows: AccessWindowLite[];
  stats: AoiStats;
}

/** Singleton store keyed by AOI id. */
export const accessByAoi = new SvelteMap<string, AoiAccessState>();

/** Reset all stored access results (called at the start of every stream). */
export function resetAccess(): void {
  accessByAoi.clear();
}

/** Append a streamed pair into the store and recompute that AOI's stats. */
export function ingestPair(p: AccessPairResult, scenarioStartMs: number, scenarioEndMs: number): void {
  const existing = accessByAoi.get(p.aoiId) ?? { windows: [], stats: emptyStats() };
  const newWindows: AccessWindowLite[] = p.windows.map((w) => ({
    startMs: Date.parse(w.startIso),
    endMs: Date.parse(w.endIso),
  }));
  const merged = existing.windows.concat(newWindows);
  accessByAoi.set(p.aoiId, {
    windows: merged,
    stats: computeStats(merged, scenarioStartMs, scenarioEndMs),
  });
}

function emptyStats(): AoiStats {
  return {
    count: 0,
    totalAccessSeconds: 0,
    meanGapSeconds: null,
    medianGapSeconds: null,
    maxGapSeconds: null,
  };
}

/**
 * Derive revisit stats from a window list and the scenario boundaries.
 * Gaps are between consecutive (sorted) window ends and the following
 * window starts; pre-first and post-last gaps are not counted.
 */
export function computeStats(
  windows: AccessWindowLite[],
  _scenarioStartMs: number,
  _scenarioEndMs: number,
): AoiStats {
  if (windows.length === 0) return emptyStats();
  const sorted = [...windows].sort((a, b) => a.startMs - b.startMs);
  const totalAccessSeconds = sorted.reduce((sum, w) => sum + (w.endMs - w.startMs), 0) / 1000;
  const gaps: number[] = [];
  for (let i = 1; i < sorted.length; i++) {
    const gap = (sorted[i].startMs - sorted[i - 1].endMs) / 1000;
    if (gap > 0) gaps.push(gap);
  }
  if (gaps.length === 0) {
    return { count: sorted.length, totalAccessSeconds, meanGapSeconds: null, medianGapSeconds: null, maxGapSeconds: null };
  }
  const sortedGaps = [...gaps].sort((a, b) => a - b);
  const mid = Math.floor(sortedGaps.length / 2);
  const median = sortedGaps.length % 2
    ? sortedGaps[mid]
    : (sortedGaps[mid - 1] + sortedGaps[mid]) / 2;
  return {
    count: sorted.length,
    totalAccessSeconds,
    meanGapSeconds: gaps.reduce((s, g) => s + g, 0) / gaps.length,
    medianGapSeconds: median,
    maxGapSeconds: Math.max(...gaps),
  };
}
