<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { scenario } from "$lib/state/scenario.svelte";
  import { runSweep, sweepRunning, resetSweep, type SweepConfig, type SweepParam, type SweepMetric } from "$lib/state/sweep.svelte";
  import SweepChart from "../SweepChart.svelte";

  let param: SweepParam = $state("satsPerPlane");
  let min = $state(2);
  let max = $state(8);
  let step = $state(1);
  let metric: SweepMetric = $state("meanGap");

  let ctl: AbortController | null = null;

  /**
   * Sensible default range/step for each sweep axis. Phasing is special: the
   * Walker F parameter is only valid in [0, P), so its max tracks the current
   * plane count.
   */
  function presetFor(p: SweepParam): { min: number; max: number; step: number } {
    switch (p) {
      case "satsPerPlane": return { min: 2, max: 10, step: 1 };
      case "planes": return { min: 1, max: 8, step: 1 };
      case "phasing": return { min: 0, max: Math.max(0, scenario.walker.p - 1), step: 1 };
      case "altitudeKm": return { min: 400, max: 1200, step: 100 };
      case "inclinationDeg": return { min: 30, max: 98, step: 8 };
    }
  }

  // When the swept parameter changes, snap the range/step to its preset. Only
  // `param` (and, for phasing, the plane count) is read here, so manual edits
  // to min/max/step persist until the next parameter change.
  $effect(() => {
    const preset = presetFor(param);
    min = preset.min;
    max = preset.max;
    step = preset.step;
  });

  const paramLabels: Record<SweepParam, string> = {
    satsPerPlane: "Sats per plane",
    planes: "Planes",
    phasing: "Phasing (F)",
    altitudeKm: "Altitude (km)",
    inclinationDeg: "Inclination (°)",
  };
  const metricLabels: Record<SweepMetric, string> = {
    meanGap: "Mean gap (s)",
    medianGap: "Median gap (s)",
    maxGap: "Max gap (s)",
    count: "Window count",
    totalAccess: "Total access (s)",
  };

  function start(): void {
    ctl?.abort();
    ctl = new AbortController();
    const cfg: SweepConfig = { param, min, max, step, metric, concurrency: 4 };
    void runSweep(scenario, cfg, ctl.signal);
  }

  /** Stop the in-flight sweep but keep whatever points have landed so far. */
  function cancel(): void {
    ctl?.abort();
  }

  /** Abort (if running) and wipe the chart. */
  function clear(): void {
    ctl?.abort();
    resetSweep();
  }

  const inputCls = "w-full bg-neutral-900 border border-neutral-700 rounded px-2 py-1 text-sm";
</script>

<div class="h-full overflow-auto p-3 space-y-3 text-xs">
  <div class="grid grid-cols-2 gap-2">
    <label>Parameter
      <select class={inputCls} bind:value={param}>
        {#each Object.entries(paramLabels) as [k, v] (k)}<option value={k}>{v}</option>{/each}
      </select>
    </label>
    <label>Metric
      <select class={inputCls} bind:value={metric}>
        {#each Object.entries(metricLabels) as [k, v] (k)}<option value={k}>{v}</option>{/each}
      </select>
    </label>
    <label>Min <input class={inputCls} type="number" bind:value={min} /></label>
    <label>Max <input class={inputCls} type="number" bind:value={max} /></label>
    <label>Step <input class={inputCls} type="number" min="0.1" step="0.1" bind:value={step} /></label>
  </div>

  <div class="flex items-center gap-2">
    <button
      type="button"
      class="px-3 py-1 rounded border border-neutral-700 bg-neutral-900 hover:bg-neutral-800"
      onclick={sweepRunning.value ? cancel : start}
    >
      {sweepRunning.value ? "Cancel" : "Run sweep"}
    </button>
    <button type="button" class="px-3 py-1 rounded border border-neutral-700 bg-neutral-900 hover:bg-neutral-800" onclick={clear}>Clear</button>
    {#if sweepRunning.value}<span class="text-cyan-400">running · {sweepRunning.done}/{sweepRunning.total}</span>{/if}
  </div>

  <SweepChart xLabel={paramLabels[param]} yLabel={metricLabels[metric]} />
</div>
