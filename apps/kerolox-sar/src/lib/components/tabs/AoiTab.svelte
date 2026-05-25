<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { accessByAoi, type AoiAccessState } from "$lib/state/access.svelte";

  let { aoiId, label }: { aoiId: string; label: string } = $props();

  const state = $derived<AoiAccessState | undefined>(accessByAoi.get(aoiId));

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

  <section class="grid grid-cols-2 gap-2 p-3 text-xs">
    <div class="text-neutral-400">Windows</div>
    <div class="font-mono text-right">{state?.stats.count ?? 0}</div>

    <div class="text-neutral-400">Total access</div>
    <div class="font-mono text-right">{fmtSeconds(state?.stats.totalAccessSeconds ?? 0)}</div>

    <div class="text-neutral-400">Mean gap</div>
    <div class="font-mono text-right">{fmtSeconds(state?.stats.meanGapSeconds ?? null)}</div>

    <div class="text-neutral-400">Median gap</div>
    <div class="font-mono text-right">{fmtSeconds(state?.stats.medianGapSeconds ?? null)}</div>

    <div class="text-neutral-400">Max gap</div>
    <div class="font-mono text-right">{fmtSeconds(state?.stats.maxGapSeconds ?? null)}</div>
  </section>

  <section class="border-t border-neutral-800">
    <table class="w-full text-xs">
      <thead class="text-neutral-400 uppercase">
        <tr class="border-b border-neutral-800">
          <th class="text-left px-3 py-2">Start</th>
          <th class="text-left px-3 py-2">End</th>
        </tr>
      </thead>
      <tbody class="text-neutral-200 font-mono">
        {#each state?.windows ?? [] as w, i (i)}
          <tr class="row-in border-b border-neutral-900/40">
            <td class="px-3 py-1">{fmtIso(w.startMs)}</td>
            <td class="px-3 py-1">{fmtIso(w.endMs)}</td>
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
