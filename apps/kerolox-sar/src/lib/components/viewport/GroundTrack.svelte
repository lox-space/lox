<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { T } from "@threlte/core";
  import { MeshLineGeometry, MeshLineMaterial } from "@threlte/extras";
  import { Origin } from "@lox-space/wasm";
  import { Vector3 } from "three";
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
  // x = r cos(lat) cos(lon), y = r sin(lat), z = -r cos(lat) sin(lon).
  const liftRadius = earthRadiusKm + 5;
  const points = $derived.by((): Vector3[] => {
    const n = traj.groundDeg.length / 2;
    const pts: Vector3[] = [];
    for (let i = 0; i < n; i++) {
      const lat = (traj.groundDeg[2 * i] * Math.PI) / 180;
      const lon = (traj.groundDeg[2 * i + 1] * Math.PI) / 180;
      const cl = Math.cos(lat);
      pts.push(new Vector3(
        liftRadius * cl * Math.cos(lon),
        liftRadius * Math.sin(lat),
        -liftRadius * cl * Math.sin(lon),
      ));
    }
    return pts;
  });
</script>

<T.Mesh>
  <MeshLineGeometry {points} />
  <MeshLineMaterial {color} attenuate={false} width={20} transparent opacity={0.6} />
</T.Mesh>
