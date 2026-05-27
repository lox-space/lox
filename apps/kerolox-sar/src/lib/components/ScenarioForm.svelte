<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  // Phase 1: this component imports and directly mutates the global `scenario`
  // runes-state. When a second scenario or component-isolation tests land in
  // a later phase, migrate to props or context injection.
  import { scenario, isWalkerValid } from "$lib/state/scenario.svelte";

  const inputCls = "mt-1 w-full bg-neutral-900 border border-neutral-700 rounded px-2 py-1 text-sm";
  const labelCls = "block text-xs uppercase tracking-wide text-neutral-400 mt-3";
  const sectionCls = "border-t border-neutral-800 pt-4 mt-4 first:border-t-0 first:pt-0 first:mt-0";
</script>

<aside class="w-72 h-full overflow-y-auto p-4 bg-neutral-950 border-r border-neutral-800 text-neutral-100">
  <section class={sectionCls}>
    <h2 class="text-sm font-semibold uppercase text-neutral-300">Scenario</h2>
    <label class={labelCls}>
      Start time (ISO 8601 UTC)
      <input class={inputCls} type="text" bind:value={scenario.startTimeIso} />
    </label>
    <label class={labelCls}>
      Duration (hours)
      <input class={inputCls} type="number" min="0.1" step="0.1" bind:value={scenario.durationHours} />
    </label>
  </section>

  <section class={sectionCls}>
    <h2 class="text-sm font-semibold uppercase text-neutral-300">Walker constellation</h2>

    <label class={labelCls}>
      Pattern
      <select class={inputCls} bind:value={scenario.walker.pattern}>
        <option value="delta">Delta (RAAN over 360°)</option>
        <option value="star">Star (RAAN over 180°)</option>
      </select>
    </label>

    <label class={labelCls}>
      Sats per plane
      <input class={inputCls} type="number" min="1" step="1" bind:value={scenario.walker.satsPerPlane} />
    </label>

    <label class={labelCls}>
      P — planes
      <input class={inputCls} type="number" min="1" step="1" bind:value={scenario.walker.p} />
    </label>

    <label class={labelCls}>
      T — total satellites (derived)
      <input
        class="{inputCls} text-neutral-400 cursor-not-allowed"
        type="number"
        disabled
        value={scenario.walker.satsPerPlane * scenario.walker.p}
      />
    </label>

    <label class={labelCls}>
      F — phasing
      <input class={inputCls} type="number" min="0" step="1" bind:value={scenario.walker.f} />
    </label>

    <label class={labelCls}>
      Altitude (km)
      <input class={inputCls} type="number" min="200" step="1" bind:value={scenario.walker.altitudeKm} />
    </label>

    <label class={labelCls}>
      Inclination (°)
      <input class={inputCls} type="number" min="0" max="180" step="0.1" bind:value={scenario.walker.inclinationDeg} />
    </label>

    {#if !isWalkerValid(scenario.walker)}
      <p class="mt-2 text-xs text-amber-400">Walker config invalid (F ∈ [0, P), positive sats per plane and altitude).</p>
    {/if}
  </section>

  <section class={sectionCls}>
    <h2 class="text-sm font-semibold uppercase text-neutral-300">SAR sensor</h2>

    <label class={labelCls}>
      Look side
      <select class={inputCls} bind:value={scenario.sar.lookSide}>
        <option value="LEFT">Left</option>
        <option value="RIGHT">Right</option>
      </select>
    </label>

    <label class={labelCls}>
      Min incidence (°)
      <input class={inputCls} type="number" step="0.1" bind:value={scenario.sar.minIncidenceDeg} />
    </label>

    <label class={labelCls}>
      Max incidence (°)
      <input class={inputCls} type="number" step="0.1" bind:value={scenario.sar.maxIncidenceDeg} />
    </label>
  </section>

  <section class={sectionCls}>
    <h2 class="text-sm font-semibold uppercase text-neutral-300">Comparison</h2>

    <label class="flex items-center gap-2 mt-3 text-sm text-neutral-200">
      <input type="checkbox" bind:checked={scenario.compareIceye} />
      Compare vs ICEYE
    </label>
    <p class="mt-1 text-xs text-neutral-500">
      Runs the fielded ICEYE fleet (real TLEs, SGP4) through the same access analysis.
    </p>
  </section>
</aside>
