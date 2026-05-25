<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import SatellitesTab from "./tabs/SatellitesTab.svelte";

  // Phase 1: only the Satellites tab exists. AOI tabs land in Phase 2.
  type Tab = "satellites";
  let active: Tab = $state("satellites");
</script>

<section class="w-[28rem] h-full flex flex-col bg-neutral-950 border-l border-neutral-800">
  <nav class="flex border-b border-neutral-800 text-xs">
    <button
      type="button"
      class="px-3 py-2 {active === 'satellites'
        ? 'text-neutral-100 border-b-2 border-cyan-400'
        : 'text-neutral-400'}"
      onclick={() => (active = "satellites")}
    >
      Satellites
    </button>
    <span class="px-3 py-2 text-neutral-600 italic">AOI tabs · Phase 2</span>
  </nav>
  <div class="flex-1 overflow-hidden">
    {#if active === "satellites"}
      <SatellitesTab />
    {/if}
  </div>
</section>
