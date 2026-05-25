<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { onMount } from "svelte";
  import ScenarioForm from "$lib/components/ScenarioForm.svelte";
  import Viewport from "$lib/components/Viewport.svelte";
  import ResultsPanel from "$lib/components/ResultsPanel.svelte";
  import StatusPill from "$lib/components/StatusPill.svelte";
  import { ensureWalkerReady, runWalker } from "$lib/walker.svelte";
  import { scenario, isWalkerValid } from "$lib/state/scenario.svelte";
  import {
    ingestPair, resetAccess,
  } from "$lib/state/access.svelte";
  import {
    markStart, markDone, markCancelled, markError, bumpPair,
  } from "$lib/state/status.svelte";
  import { runComputeAccess } from "$lib/rpc/client";
  import type { AccessRequest } from "@kerolox/proto-ts";

  let ready = $state(false);

  onMount(async () => {
    await ensureWalkerReady();
    ready = true;
  });

  // Debounced reactive runner: re-runs ComputeAccess whenever scenario changes.
  $effect(() => {
    if (!ready) return;
    if (!isWalkerValid(scenario.walker)) return;
    const satellites = runWalker(scenario);
    if (satellites.length === 0) return;

    const ctl = new AbortController();
    let cancelled = false;

    const timer = setTimeout(async () => {
      if (cancelled) return;
      resetAccess();
      const scenarioStartMs = Date.parse(scenario.startTimeIso);
      const scenarioEndMs = scenarioStartMs + scenario.durationHours * 3600 * 1000;
      const req: AccessRequest = {
        startTimeIso: scenario.startTimeIso,
        durationSeconds: scenario.durationHours * 3600,
        satellites: satellites.map((s) => ({
          id: `p${s.plane}-s${s.indexInPlane}`,
          smaM: s.smaM,
          ecc: s.ecc,
          incRad: s.incRad,
          raanRad: s.raanRad,
          aopRad: s.aopRad,
          trueAnomalyRad: s.trueAnomalyRad,
          plane: s.plane,
          indexInPlane: s.indexInPlane,
        })) as unknown as AccessRequest["satellites"],
        sar: {
          lookSide: scenario.sar.lookSide === "LEFT" ? 1 : 2,
          minIncidenceDeg: scenario.sar.minIncidenceDeg,
          maxIncidenceDeg: scenario.sar.maxIncidenceDeg,
        } as unknown as AccessRequest["sar"],
        aoiIds: ["hormuz", "black_sea"],
        comparators: [],
        stepSeconds: 30,
      } as unknown as AccessRequest;

      const pairsExpected = satellites.length * 2; // 2 AOIs
      await runComputeAccess(req, {
        onStart: () => markStart(pairsExpected),
        onPair: (p) => {
          ingestPair(p, scenarioStartMs, scenarioEndMs);
          bumpPair();
        },
        onDone: (ms) => markDone(ms),
        onCancel: () => markCancelled(),
        onError: (err) => markError(err.message),
      }, ctl.signal);
    }, 300);

    return () => {
      cancelled = true;
      clearTimeout(timer);
      ctl.abort();
    };
  });
</script>

<div class="h-full flex flex-col">
  <header class="flex items-center justify-between px-4 py-2 bg-neutral-950 border-b border-neutral-800">
    <h1 class="text-sm font-semibold uppercase tracking-wide">Kerolox · SAR Constellation Sizing</h1>
    <StatusPill />
  </header>
  {#if ready}
    <main class="flex-1 flex min-h-0">
      <ScenarioForm />
      <Viewport />
      <ResultsPanel />
    </main>
  {:else}
    <main class="flex-1 flex items-center justify-center text-neutral-400 text-sm">
      Loading WASM module…
    </main>
  {/if}
</div>
