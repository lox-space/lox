<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { sweepPoints, type SweepPoint } from "$lib/state/sweep.svelte";

  let { xLabel = "", yLabel = "" }: { xLabel?: string; yLabel?: string } = $props();

  const W = 480;
  const H = 280;
  const PAD = 40;

  const series = $derived(Array.from(sweepPoints.entries()));

  const bounds = $derived.by(() => {
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const [, pts] of series) {
      for (const p of pts) {
        minX = Math.min(minX, p.x); maxX = Math.max(maxX, p.x);
        minY = Math.min(minY, p.y); maxY = Math.max(maxY, p.y);
      }
    }
    if (!Number.isFinite(minX)) { minX = 0; maxX = 1; minY = 0; maxY = 1; }
    if (minY === maxY) { maxY = minY + 1; }
    if (minX === maxX) { maxX = minX + 1; }
    return { minX, maxX, minY, maxY };
  });

  const COLORS: Record<string, string> = { hormuz: "#7c8aff", black_sea: "#34d399" };

  function sx(x: number): number {
    const { minX, maxX } = bounds;
    return PAD + ((x - minX) / (maxX - minX)) * (W - 2 * PAD);
  }
  function sy(y: number): number {
    const { minY, maxY } = bounds;
    return H - PAD - ((y - minY) / (maxY - minY)) * (H - 2 * PAD);
  }
  function path(pts: SweepPoint[]): string {
    return pts.map((p, i) => `${i === 0 ? "M" : "L"}${sx(p.x).toFixed(1)},${sy(p.y).toFixed(1)}`).join(" ");
  }
</script>

<svg viewBox="0 0 {W} {H}" class="w-full h-auto text-neutral-400">
  <!-- axes -->
  <line x1={PAD} y1={H - PAD} x2={W - PAD} y2={H - PAD} stroke="currentColor" stroke-width="1" />
  <line x1={PAD} y1={PAD} x2={PAD} y2={H - PAD} stroke="currentColor" stroke-width="1" />
  <text x={W / 2} y={H - 6} text-anchor="middle" font-size="11" fill="currentColor">{xLabel}</text>
  <text x="12" y={H / 2} text-anchor="middle" font-size="11" fill="currentColor" transform="rotate(-90 12 {H / 2})">{yLabel}</text>

  <!-- bounds labels -->
  <text x={PAD} y={H - PAD + 14} font-size="9" fill="currentColor">{bounds.minX.toFixed(0)}</text>
  <text x={W - PAD} y={H - PAD + 14} text-anchor="end" font-size="9" fill="currentColor">{bounds.maxX.toFixed(0)}</text>
  <text x={PAD - 4} y={H - PAD} text-anchor="end" font-size="9" fill="currentColor">{bounds.minY.toFixed(0)}</text>
  <text x={PAD - 4} y={PAD + 8} text-anchor="end" font-size="9" fill="currentColor">{bounds.maxY.toFixed(0)}</text>

  {#each series as [aoiId, pts] (aoiId)}
    {@const color = COLORS[aoiId] ?? "#aaa"}
    <path d={path(pts)} fill="none" stroke={color} stroke-width="2" />
    {#each pts as p (p.x)}
      <circle cx={sx(p.x)} cy={sy(p.y)} r="3" fill={color} />
    {/each}
  {/each}
</svg>
