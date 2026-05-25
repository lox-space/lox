<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import type { StatusState } from "$lib/state/status.svelte";

  let { state, label = "" }: { state: StatusState; label?: string } = $props();

  const dotClass = $derived(
    {
      idle: "bg-neutral-500",
      computing: "bg-cyan-400 animate-pulse",
      done: "bg-emerald-400",
      error: "bg-rose-400",
      cancelled: "bg-amber-400",
    }[state.status],
  );

  const text = $derived.by(() => {
    switch (state.status) {
      case "idle":
        return "idle";
      case "computing":
        return state.expected > 0
          ? `computing · ${state.received}/${state.expected}`
          : "computing";
      case "done":
        if (state.lastDurationMs == null) return "done";
        return `done · ${(state.lastDurationMs / 1000).toFixed(1)} s`;
      case "error":
        return `error · ${state.lastError ?? "unknown"}`;
      case "cancelled":
        return "cancelled";
    }
  });
</script>

<span
  class="inline-flex items-center gap-1 px-2 py-0.5 text-xs rounded-full border border-neutral-700 bg-neutral-900 text-neutral-300"
>
  <span class="size-2 rounded-full {dotClass} transition-colors duration-150"></span>
  {#if label}<span class="text-neutral-500 uppercase tracking-wide">{label}</span>{/if}
  {text}
</span>
