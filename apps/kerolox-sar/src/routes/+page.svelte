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
  import { ensureWalkerReady } from "$lib/walker.svelte";

  let ready = $state(false);

  onMount(async () => {
    await ensureWalkerReady();
    ready = true;
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
