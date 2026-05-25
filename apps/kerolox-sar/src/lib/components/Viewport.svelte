<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { onMount } from "svelte";
  import GlobeView from "./viewport/GlobeView.svelte";
  import MapView from "./viewport/MapView.svelte";
  import Transport from "./Transport.svelte";
  import { loadAois, type AoiPolygon } from "$lib/aois";

  type View = "globe" | "map";
  let active: View = $state("globe");
  let aois: Map<string, AoiPolygon> = $state(new Map());

  onMount(async () => {
    aois = await loadAois();
  });
</script>

<section class="flex-1 h-full flex flex-col bg-neutral-900 border-r border-neutral-800">
  <header class="px-3 py-1.5 flex items-center justify-between border-b border-neutral-800 text-xs text-neutral-400">
    <span>Viewport</span>
    <div class="inline-flex border border-neutral-700 rounded overflow-hidden">
      <button
        type="button"
        class="px-3 py-1 {active === 'globe' ? 'bg-neutral-700 text-neutral-100' : 'bg-neutral-900 text-neutral-300'}"
        onclick={() => (active = "globe")}
      >
        Globe
      </button>
      <button
        type="button"
        class="px-3 py-1 {active === 'map' ? 'bg-neutral-700 text-neutral-100' : 'bg-neutral-900 text-neutral-300'}"
        onclick={() => (active = "map")}
      >
        Map
      </button>
    </div>
  </header>
  {#if active === "globe"}
    <GlobeView {aois} />
  {:else}
    <MapView {aois} />
  {/if}
  <Transport />
</section>
