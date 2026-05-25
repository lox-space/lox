<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { status } from "$lib/state/status.svelte";

  const dotClass = $derived(
    {
      idle: "bg-neutral-500",
      computing: "bg-cyan-400 animate-pulse",
      done: "bg-emerald-400",
      error: "bg-rose-400",
      cancelled: "bg-amber-400",
    }[status.status],
  );

  const label = $derived.by(() => {
    switch (status.status) {
      case "idle":
        return "idle";
      case "computing":
        return status.pairsExpected > 0
          ? `computing · ${status.pairsReceived}/${status.pairsExpected}`
          : "computing";
      case "done":
        if (status.lastDurationMs == null) return "done";
        return `done · ${(status.lastDurationMs / 1000).toFixed(1)} s`;
      case "error":
        return `error · ${status.lastError ?? "unknown"}`;
      case "cancelled":
        return "cancelled";
    }
  });
</script>

<span
  class="inline-flex items-center gap-1 px-2 py-0.5 text-xs rounded-full border border-neutral-700 bg-neutral-900 text-neutral-300"
>
  <span class="size-2 rounded-full {dotClass} transition-colors duration-150"></span>
  {label}
</span>
