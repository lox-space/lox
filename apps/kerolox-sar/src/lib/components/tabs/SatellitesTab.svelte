<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { scenario } from "$lib/state/scenario.svelte";
  import { runWalker, type SatelliteElements } from "$lib/walker.svelte";

  // Re-runs whenever scenario changes thanks to the runes graph.
  const satellites = $derived<SatelliteElements[]>(runWalker(scenario));

  function rad(x: number): string {
    return ((x * 180) / Math.PI).toFixed(2);
  }
</script>

{#if satellites.length === 0}
  <p class="p-4 text-sm text-neutral-400">No satellites — check the Walker configuration.</p>
{:else}
  <div class="overflow-auto" data-testid="satellites-table">
    <table class="w-full text-xs">
      <thead class="text-neutral-400 uppercase">
        <tr class="border-b border-neutral-800">
          <th class="text-left px-3 py-2">Plane</th>
          <th class="text-left px-3 py-2">Idx</th>
          <th class="text-right px-3 py-2">SMA (km)</th>
          <th class="text-right px-3 py-2">Ecc</th>
          <th class="text-right px-3 py-2">Inc (°)</th>
          <th class="text-right px-3 py-2">RAAN (°)</th>
          <th class="text-right px-3 py-2">AOP (°)</th>
          <th class="text-right px-3 py-2">TA (°)</th>
        </tr>
      </thead>
      <tbody class="text-neutral-200 font-mono">
        {#each satellites as sat (sat.plane * 1000 + sat.indexInPlane)}
          <tr class="border-b border-neutral-900/40 hover:bg-neutral-900/50">
            <td class="px-3 py-1">{sat.plane}</td>
            <td class="px-3 py-1">{sat.indexInPlane}</td>
            <td class="px-3 py-1 text-right">{(sat.smaM / 1000).toFixed(1)}</td>
            <td class="px-3 py-1 text-right">{sat.ecc.toFixed(4)}</td>
            <td class="px-3 py-1 text-right">{rad(sat.incRad)}</td>
            <td class="px-3 py-1 text-right">{rad(sat.raanRad)}</td>
            <td class="px-3 py-1 text-right">{rad(sat.aopRad)}</td>
            <td class="px-3 py-1 text-right">{rad(sat.trueAnomalyRad)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
