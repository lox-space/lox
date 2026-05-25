<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { scenario } from "$lib/state/scenario.svelte";
</script>

<section class="flex-1 h-full flex flex-col bg-neutral-900 border-r border-neutral-800">
  <!-- Phase 1 placeholder body; Phase 3 replaces with Globe / Map viewports. -->
  <header class="px-4 py-2 text-sm text-neutral-400 border-b border-neutral-800">
    Viewport
  </header>
  <pre
    class="flex-1 overflow-auto p-4 text-xs leading-relaxed text-neutral-400 whitespace-pre">{JSON.stringify(
      scenario,
      null,
      2,
    )}</pre>
</section>
