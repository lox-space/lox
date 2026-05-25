<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import SatellitesTab from "./tabs/SatellitesTab.svelte";
  import AoiTab from "./tabs/AoiTab.svelte";

  type Tab = "satellites" | "hormuz" | "black_sea";
  let active: Tab = $state("satellites");

  const tabs: { id: Tab; label: string }[] = [
    { id: "satellites", label: "Satellites" },
    { id: "hormuz", label: "Hormuz" },
    { id: "black_sea", label: "Black Sea" },
  ];
</script>

<section class="w-[34rem] h-full flex flex-col bg-neutral-950 border-l border-neutral-800">
  <nav class="flex border-b border-neutral-800 text-xs">
    {#each tabs as t (t.id)}
      <button
        type="button"
        class="px-3 py-2 {active === t.id
          ? 'text-neutral-100 border-b-2 border-cyan-400'
          : 'text-neutral-400 hover:text-neutral-200'}"
        onclick={() => (active = t.id)}
      >
        {t.label}
      </button>
    {/each}
  </nav>
  <div class="flex-1 overflow-hidden">
    {#if active === "satellites"}
      <SatellitesTab />
    {:else if active === "hormuz"}
      <AoiTab aoiId="hormuz" label="Strait of Hormuz" />
    {:else if active === "black_sea"}
      <AoiTab aoiId="black_sea" label="Black Sea" />
    {/if}
  </div>
</section>
