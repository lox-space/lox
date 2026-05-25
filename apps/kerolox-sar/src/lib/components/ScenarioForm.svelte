<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { scenario, isWalkerValid } from "$lib/state/scenario.svelte";

  const inputCls = "w-full bg-neutral-900 border border-neutral-700 rounded px-2 py-1 text-sm";
  const labelCls = "block text-xs uppercase tracking-wide text-neutral-400 mt-3 mb-1";
  const sectionCls = "border-t border-neutral-800 pt-4 mt-4 first:border-t-0 first:pt-0 first:mt-0";
</script>

<aside class="w-72 h-full overflow-y-auto p-4 bg-neutral-950 border-r border-neutral-800 text-neutral-100">
  <section class={sectionCls}>
    <h2 class="text-sm font-semibold uppercase text-neutral-300">Scenario</h2>
    <label class={labelCls}>Start time (ISO 8601 UTC)</label>
    <input class={inputCls} type="text" bind:value={scenario.startTimeIso} />

    <label class={labelCls}>Duration (hours)</label>
    <input class={inputCls} type="number" min="0.1" step="0.1" bind:value={scenario.durationHours} />
  </section>

  <section class={sectionCls}>
    <h2 class="text-sm font-semibold uppercase text-neutral-300">Walker delta</h2>

    <label class={labelCls}>T — total satellites</label>
    <input class={inputCls} type="number" min="1" step="1" bind:value={scenario.walker.t} />

    <label class={labelCls}>P — planes</label>
    <input class={inputCls} type="number" min="1" step="1" bind:value={scenario.walker.p} />

    <label class={labelCls}>F — phasing</label>
    <input class={inputCls} type="number" min="0" step="1" bind:value={scenario.walker.f} />

    <label class={labelCls}>Altitude (km)</label>
    <input class={inputCls} type="number" min="200" step="1" bind:value={scenario.walker.altitudeKm} />

    <label class={labelCls}>Inclination (°)</label>
    <input class={inputCls} type="number" step="0.1" bind:value={scenario.walker.inclinationDeg} />

    {#if !isWalkerValid(scenario.walker)}
      <p class="mt-2 text-xs text-amber-400">Walker config invalid (T must divide by P, F ∈ [0, P), positive altitude).</p>
    {/if}
  </section>

  <section class={sectionCls}>
    <h2 class="text-sm font-semibold uppercase text-neutral-300">SAR sensor</h2>

    <label class={labelCls}>Look side</label>
    <select class={inputCls} bind:value={scenario.sar.lookSide}>
      <option value="LEFT">Left</option>
      <option value="RIGHT">Right</option>
    </select>

    <label class={labelCls}>Min incidence (°)</label>
    <input class={inputCls} type="number" step="0.1" bind:value={scenario.sar.minIncidenceDeg} />

    <label class={labelCls}>Max incidence (°)</label>
    <input class={inputCls} type="number" step="0.1" bind:value={scenario.sar.maxIncidenceDeg} />
  </section>
</aside>
