<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { T } from "@threlte/core";
  import { Origin } from "@lox-space/wasm";
  import { BufferGeometry, Float32BufferAttribute, LineBasicMaterial } from "three";
  import type { SampledTrajectoryView } from "$lib/state/trajectories.svelte";

  let { traj, color = "#7c8aff" }: {
    traj: SampledTrajectoryView;
    color?: string;
  } = $props();

  const earth = new Origin("Earth");
  const earthRadiusKm = earth.mean_radius() / 1000;
  $effect(() => () => { earth.free(); });

  // Lift each (lat, lon) sample onto a sphere just above the surface to
  // avoid z-fighting with the Earth mesh. Three.js convention (Y-up):
  // x = r cos(lat) cos(-lon), y = r sin(lat), z = -r cos(lat) sin(-lon).
  const liftRadius = earthRadiusKm + 5;
  const points = $derived.by((): Float32Array => {
    const n = traj.groundDeg.length / 2;
    const out = new Float32Array(n * 3);
    for (let i = 0; i < n; i++) {
      const lat = (traj.groundDeg[2 * i] * Math.PI) / 180;
      const lon = (traj.groundDeg[2 * i + 1] * Math.PI) / 180;
      const cl = Math.cos(lat);
      out[3 * i] = liftRadius * cl * Math.cos(lon);
      out[3 * i + 1] = liftRadius * Math.sin(lat);
      out[3 * i + 2] = -liftRadius * cl * Math.sin(lon);
    }
    return out;
  });

  const geometry = $derived.by(() => {
    const g = new BufferGeometry();
    g.setAttribute("position", new Float32BufferAttribute(points, 3));
    return g;
  });

  const material = $derived.by(() => new LineBasicMaterial({ color, transparent: true, opacity: 0.45 }));

  $effect(() => {
    // When `geometry` or `material` is replaced (because `traj` changed)
    // or when the component unmounts, dispose the previous Three.js
    // resources to avoid leaking GPU memory.
    const g = geometry;
    const m = material;
    return () => {
      g.dispose();
      m.dispose();
    };
  });
</script>

<T.Line args={[geometry, material]} />
