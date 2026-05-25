<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { onMount, untrack } from "svelte";
  import ScenarioForm from "$lib/components/ScenarioForm.svelte";
  import Viewport from "$lib/components/Viewport.svelte";
  import ResultsPanel from "$lib/components/ResultsPanel.svelte";
  import StatusPill from "$lib/components/StatusPill.svelte";
  import { ensureWalkerReady, runWalker } from "$lib/walker.svelte";
  import { scenario, isWalkerValid } from "$lib/state/scenario.svelte";
  import {
    ingestPair, resetAccess,
  } from "$lib/state/access.svelte";
  import { accessStatus, propagationStatus } from "$lib/state/status.svelte";
  import { runComputeAccess } from "$lib/rpc/client";
  import type { AccessRequest } from "@kerolox/proto-ts";
  import { ensureTrajectories } from "$lib/state/trajectories.svelte";
  import { setBounds, seek, pause } from "$lib/state/playback.svelte";

  let ready = $state(false);

  onMount(async () => {
    // Set initial playback bounds eagerly from the current scenario so
    // the Transport clock shows the scenario start time immediately,
    // even before the trajectory cache populates.
    const initialStart = Date.parse(scenario.startTimeIso);
    const initialEnd = initialStart + scenario.durationHours * 3600 * 1000;
    if (Number.isFinite(initialStart) && Number.isFinite(initialEnd)) {
      setBounds(initialStart, initialEnd);
      seek(initialStart);
    }
    await ensureWalkerReady();
    ready = true;
  });

  // Phase 3: populate the trajectory cache and reset the playback bounds
  // whenever the scenario or Walker output changes. Independent from the
  // ComputeAccess effect below — the runes graph dedupes the actual
  // Walker work since runWalker is pure.
  $effect(() => {
    if (!ready) return;
    if (!isWalkerValid(scenario.walker)) return;
    const satellites = runWalker(scenario);
    if (satellites.length === 0) return;

    const scenarioStartMs = Date.parse(scenario.startTimeIso);
    const scenarioEndMs = scenarioStartMs + scenario.durationHours * 3600 * 1000;

    // Wrap the playback mutations in untrack so the reads inside
    // setBounds/seek (for the bounds-snap and clamping logic) aren't
    // tracked as dependencies of this effect — otherwise seek() writing
    // playback.currentTime would re-trigger the effect via the read in
    // setBounds, infinitely.
    untrack(() => {
      setBounds(scenarioStartMs, scenarioEndMs);
      seek(scenarioStartMs);
      pause();
    });

    // Debounce the propagation RPC so rapid form edits (typing) don't
    // thrash the engine with cancel/restart cycles. ensureTrajectories has
    // its own AbortController, so a fired-then-superseded run is cancelled
    // cleanly; the debounce keeps it from firing at all until typing stops.
    const timer = setTimeout(() => {
      ensureTrajectories(scenario, satellites);
    }, 300);

    return () => clearTimeout(timer);
  });

  // Reactive runner: re-runs ComputeAccess whenever scenario changes.
  $effect(() => {
    if (!ready) return;
    if (!isWalkerValid(scenario.walker)) return;
    const satellites = runWalker(scenario);
    if (satellites.length === 0) return;

    // Read remaining scenario fields synchronously so they register as effect
    // dependencies.
    const startTimeIso = scenario.startTimeIso;
    const durationHours = scenario.durationHours;
    const sarLookSide = scenario.sar.lookSide;
    const sarMinIncidenceDeg = scenario.sar.minIncidenceDeg;
    const sarMaxIncidenceDeg = scenario.sar.maxIncidenceDeg;

    const ctl = new AbortController();

    // Debounce the ComputeAccess RPC so typing into form fields doesn't
    // fire-and-cancel a stream on every keystroke (which surfaced as a
    // flashing "cancelled" pill). The timer resets on each scenario change;
    // the RPC only fires once edits settle for 300 ms.
    const timer = setTimeout(() => {
      resetAccess();
      const scenarioStartMs = Date.parse(startTimeIso);
      const scenarioEndMs = scenarioStartMs + durationHours * 3600 * 1000;
      const req: AccessRequest = {
        startTimeIso,
        durationSeconds: durationHours * 3600,
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
          lookSide: sarLookSide === "LEFT" ? 1 : 2,
          minIncidenceDeg: sarMinIncidenceDeg,
          maxIncidenceDeg: sarMaxIncidenceDeg,
        } as unknown as AccessRequest["sar"],
        aoiIds: ["hormuz", "black_sea"],
        comparators: [],
        stepSeconds: 30,
      } as unknown as AccessRequest;

      const pairsExpected = satellites.length * 2; // 2 AOIs
      void runComputeAccess(req, {
        onStart: () => accessStatus.markStart(pairsExpected),
        onPair: (p) => {
          ingestPair(p, scenarioStartMs, scenarioEndMs);
          accessStatus.bump();
        },
        onDone: (ms) => accessStatus.markDone(ms),
        onCancel: () => accessStatus.markCancelled(),
        onError: (err) => accessStatus.markError(err.message),
      }, ctl.signal);
    }, 300);

    return () => {
      clearTimeout(timer);
      ctl.abort();
    };
  });
</script>

<div class="h-full flex flex-col">
  <header class="flex items-center justify-between px-4 py-2 bg-neutral-950 border-b border-neutral-800">
    <h1 class="text-sm font-semibold uppercase tracking-wide">Kerolox · SAR Constellation Sizing</h1>
    <div class="flex items-center gap-2">
      <StatusPill state={propagationStatus.state} label="prop" />
      <StatusPill state={accessStatus.state} label="access" />
    </div>
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
