<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { accessByAoi, type AoiAccessState, type PassDirection } from "$lib/state/access.svelte";

  let { aoiId, label }: { aoiId: string; label: string } = $props();

  /** Compact glyph for the pass direction. ↑ ascending, ↓ descending. */
  function fmtDirection(d: PassDirection): string {
    if (d === "ascending") return "↑ Asc";
    if (d === "descending") return "↓ Desc";
    return "—";
  }

  /** Format a spacecraft id "p0-s0" as the 1-indexed "1-1" used elsewhere. */
  function fmtSat(scId: string): string {
    const m = scId.match(/^p(\d+)-s(\d+)$/);
    if (!m) return scId;
    return `${parseInt(m[1], 10) + 1}-${parseInt(m[2], 10) + 1}`;
  }

  const state = $derived<AoiAccessState | undefined>(accessByAoi.get(aoiId));
  const hasComparator = $derived((state?.comparatorStats.count ?? 0) > 0);

  function fmtSeconds(s: number | null): string {
    if (s == null) return "—";
    if (s >= 3600) return `${(s / 3600).toFixed(2)} h`;
    if (s >= 60) return `${(s / 60).toFixed(1)} m`;
    return `${s.toFixed(0)} s`;
  }

  function fmtIso(ms: number): string {
    return new Date(ms).toISOString().replace("T", " ").slice(0, 19);
  }
</script>

<div class="h-full overflow-auto" data-testid="aoi-tab-{aoiId}">
  <header class="px-3 py-2 border-b border-neutral-800 text-sm text-neutral-300">
    {label}
  </header>

  <section class="grid {hasComparator ? 'grid-cols-3' : 'grid-cols-2'} gap-2 p-3 text-xs">
    {#if hasComparator}
      <div></div>
      <div class="text-right text-neutral-400 uppercase">You</div>
      <div class="text-right text-amber-400 uppercase">ICEYE</div>
    {/if}

    <div class="text-neutral-400">Windows</div>
    <div class="font-mono text-right">{state?.userStats.count ?? 0}</div>
    {#if hasComparator}<div class="font-mono text-right text-amber-300">{state?.comparatorStats.count ?? 0}</div>{/if}

    <div class="text-neutral-400">Total access</div>
    <div class="font-mono text-right">{fmtSeconds(state?.userStats.totalAccessSeconds ?? 0)}</div>
    {#if hasComparator}<div class="font-mono text-right text-amber-300">{fmtSeconds(state?.comparatorStats.totalAccessSeconds ?? 0)}</div>{/if}

    <div class="text-neutral-400">Mean gap</div>
    <div class="font-mono text-right">{fmtSeconds(state?.userStats.meanGapSeconds ?? null)}</div>
    {#if hasComparator}<div class="font-mono text-right text-amber-300">{fmtSeconds(state?.comparatorStats.meanGapSeconds ?? null)}</div>{/if}

    <div class="text-neutral-400">Median gap</div>
    <div class="font-mono text-right">{fmtSeconds(state?.userStats.medianGapSeconds ?? null)}</div>
    {#if hasComparator}<div class="font-mono text-right text-amber-300">{fmtSeconds(state?.comparatorStats.medianGapSeconds ?? null)}</div>{/if}

    <div class="text-neutral-400">Max gap</div>
    <div class="font-mono text-right">{fmtSeconds(state?.userStats.maxGapSeconds ?? null)}</div>
    {#if hasComparator}<div class="font-mono text-right text-amber-300">{fmtSeconds(state?.comparatorStats.maxGapSeconds ?? null)}</div>{/if}
  </section>

  <section class="border-t border-neutral-800">
    <table class="w-full text-xs">
      <thead class="text-neutral-400 uppercase">
        <tr class="border-b border-neutral-800">
          <th class="text-left px-3 py-2">Sat</th>
          <th class="text-left px-3 py-2">Start</th>
          <th class="text-left px-3 py-2">End</th>
          <th class="text-left px-3 py-2">Dir</th>
        </tr>
      </thead>
      <tbody class="text-neutral-200 font-mono">
        {#each state?.userWindows ?? [] as w, i (i)}
          <tr class="row-in border-b border-neutral-900/40">
            <td class="px-3 py-1">{fmtSat(w.scId)}</td>
            <td class="px-3 py-1">{fmtIso(w.startMs)}</td>
            <td class="px-3 py-1">{fmtIso(w.endMs)}</td>
            <td class="px-3 py-1" title={w.direction}>{fmtDirection(w.direction)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </section>
</div>

<style>
  .row-in {
    animation: rowIn 200ms ease-out;
  }
  @keyframes rowIn {
    from { opacity: 0; transform: translateY(-4px); }
    to   { opacity: 1; transform: translateY(0); }
  }
</style>
