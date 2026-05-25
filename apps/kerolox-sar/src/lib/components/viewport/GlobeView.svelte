<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { T, Canvas } from "@threlte/core";
  import { OrbitControls } from "@threlte/extras";
  import { WebGLRenderer } from "three";
  import { earthRotationAngleRad } from "@lox-space/wasm";
  import { Earth } from "@lox-space/threlte";
  import SatelliteMarker from "./SatelliteMarker.svelte";
  import GroundTrack from "./GroundTrack.svelte";
  import AoiPolygon from "./AoiPolygon.svelte";
  import TickAdvancer from "./TickAdvancer.svelte";
  import { trajectoryById } from "$lib/state/trajectories.svelte";
  import { playback } from "$lib/state/playback.svelte";
  import type { AoiPolygon as AoiPolygonData } from "$lib/aois";
  import { colorForPlane, parsePlaneFromId } from "./colors";

  let { aois }: { aois: Map<string, AoiPolygonData> } = $props();

  const earthRotation = $derived.by(() => {
    if (!Number.isFinite(playback.currentTime) || playback.currentTime === 0) return 0;
    const iso = new Date(playback.currentTime).toISOString();
    try {
      return earthRotationAngleRad(iso);
    } catch {
      return 0;
    }
  });
</script>

<div class="flex-1 min-h-0 relative">
  <Canvas
    createRenderer={(canvas) => new WebGLRenderer({ canvas, logarithmicDepthBuffer: true })}
  >
    <TickAdvancer />

    <T.PerspectiveCamera makeDefault position={[0, 0, 30000]} far={1e9}>
      <OrbitControls />
    </T.PerspectiveCamera>

    <T.AmbientLight intensity={2} />

    <T.Group rotation={[0, earthRotation, 0]}>
      <Earth textureUrl="/assets/Earth-color.jpg" />
      {#each Array.from(aois.values()) as a (a.id)}
        <AoiPolygon aoi={a} />
      {/each}
      {#each Array.from(trajectoryById.entries()) as [id, traj] (id)}
        <GroundTrack {traj} color={colorForPlane(parsePlaneFromId(id))} />
      {/each}
    </T.Group>

    {#each Array.from(trajectoryById.entries()) as [id, traj] (id)}
      <SatelliteMarker {traj} color={colorForPlane(parsePlaneFromId(id))} />
    {/each}
  </Canvas>
</div>
