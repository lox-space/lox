<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { playback, play, pause, seek, setRate } from "$lib/state/playback.svelte";

  const RATES = [1, 10, 60, 600];

  function toggle(): void {
    if (playback.playing) pause();
    else play();
  }

  function fmtTime(ms: number): string {
    if (!Number.isFinite(ms)) return "—";
    return new Date(ms).toISOString().replace("T", " ").slice(0, 19);
  }
</script>

<section class="border-t border-neutral-800 bg-neutral-950 px-3 py-2 flex items-center gap-3 text-xs text-neutral-200">
  <button
    type="button"
    class="px-3 py-1 rounded border border-neutral-700 bg-neutral-900 hover:bg-neutral-800"
    onclick={toggle}
  >
    {playback.playing ? "⏸ Pause" : "▶ Play"}
  </button>

  <span class="font-mono text-neutral-300 tabular-nums" data-testid="transport-clock">
    {fmtTime(playback.currentTime)}
  </span>

  <input
    type="range"
    class="flex-1"
    min={playback.startMs}
    max={playback.endMs}
    step={1000}
    value={playback.currentTime}
    oninput={(e) => seek(parseFloat((e.target as HTMLInputElement).value))}
  />

  <label class="flex items-center gap-2">
    <span class="uppercase text-neutral-400">Rate</span>
    <select
      class="bg-neutral-900 border border-neutral-700 rounded px-2 py-1"
      value={playback.rate}
      onchange={(e) => setRate(parseFloat((e.target as HTMLSelectElement).value))}
    >
      {#each RATES as r}
        <option value={r}>{r}×</option>
      {/each}
    </select>
  </label>
</section>
