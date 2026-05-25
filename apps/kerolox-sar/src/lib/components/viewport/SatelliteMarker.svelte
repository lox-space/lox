<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { T } from "@threlte/core";
  import type { SampledTrajectoryView } from "$lib/state/trajectories.svelte";
  import { playback } from "$lib/state/playback.svelte";

  let { traj, color = "#7c8aff", radius = 80 }: {
    traj: SampledTrajectoryView;
    color?: string;
    radius?: number;
  } = $props();

  const position = $derived.by((): [number, number, number] => {
    const epochs = traj.epochsMs;
    const eci = traj.eciKm;
    if (epochs.length === 0) return [0, 0, 0];
    const t = playback.currentTime;
    // Binary search for the segment containing t (assumes monotonic).
    let lo = 0;
    let hi = epochs.length - 1;
    while (hi - lo > 1) {
      const mid = (lo + hi) >> 1;
      if (epochs[mid] <= t) lo = mid;
      else hi = mid;
    }
    if (t <= epochs[0]) return [eci[0], eci[1], eci[2]];
    if (t >= epochs[epochs.length - 1]) {
      const last = epochs.length - 1;
      return [eci[3 * last], eci[3 * last + 1], eci[3 * last + 2]];
    }
    const t0 = epochs[lo];
    const t1 = epochs[hi];
    const f = t1 === t0 ? 0 : (t - t0) / (t1 - t0);
    const i0 = 3 * lo;
    const i1 = 3 * hi;
    return [
      eci[i0] + (eci[i1] - eci[i0]) * f,
      eci[i0 + 1] + (eci[i1 + 1] - eci[i0 + 1]) * f,
      eci[i0 + 2] + (eci[i1 + 2] - eci[i0 + 2]) * f,
    ];
  });
</script>

<T.Mesh {position}>
  <T.SphereGeometry args={[radius, 12, 12]} />
  <T.MeshBasicMaterial {color} />
</T.Mesh>
