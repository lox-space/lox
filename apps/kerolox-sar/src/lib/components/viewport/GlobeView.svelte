<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { T, Canvas } from "@threlte/core";
  import { OrbitControls } from "@threlte/extras";
  import { WebGLRenderer } from "three";
  import { earthRotationAngleRad, sunDirectionEci } from "@lox-space/wasm";
  import { Earth } from "@lox-space/threlte";
  import SatelliteMarker from "./SatelliteMarker.svelte";
  import GroundTrack from "./GroundTrack.svelte";
  import AoiPolygon from "./AoiPolygon.svelte";
  import TickAdvancer from "./TickAdvancer.svelte";
  import { trajectoryById, comparatorTrajectoryById } from "$lib/state/trajectories.svelte";
  import { playback } from "$lib/state/playback.svelte";
  import type { AoiPolygon as AoiPolygonData } from "$lib/aois";
  import { colorForPlane, parsePlaneFromId } from "./colors";

  /** Fixed amber for the fielded ICEYE comparator fleet, distinct from the
   *  per-plane palette used for the user's design. */
  const COMPARATOR_COLOR = "#ffaa44";

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

  // Unit Sun direction in the inertial ECI frame (Three.js Y-up), from the
  // analytical lox-earth ephemeris. Drives the directional light so the globe
  // shows a day/night terminator that the Earth rotates under. Placed far out
  // along the Sun vector; for a directional light only the direction matters.
  const SUN_DISTANCE_KM = 1.5e8;
  const sunPosition = $derived.by((): [number, number, number] => {
    const fallback: [number, number, number] = [SUN_DISTANCE_KM, 0, 0];
    if (!Number.isFinite(playback.currentTime) || playback.currentTime === 0) return fallback;
    try {
      const d = sunDirectionEci(new Date(playback.currentTime).toISOString());
      return [d[0] * SUN_DISTANCE_KM, d[1] * SUN_DISTANCE_KM, d[2] * SUN_DISTANCE_KM];
    } catch {
      return fallback;
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

    <!-- Low ambient keeps the night side dimly visible; the directional Sun
         light (inertial, outside the rotating Earth group) carves the
         terminator across the day side. -->
    <T.AmbientLight intensity={0.05} />
    <T.DirectionalLight position={sunPosition} intensity={6} />

    <T.Group rotation={[0, earthRotation, 0]}>
      <Earth textureUrl="/assets/Earth-color.jpg" />
      {#each Array.from(aois.values()) as a (a.id)}
        <AoiPolygon aoi={a} />
      {/each}
      {#each Array.from(trajectoryById.entries()) as [id, traj] (id)}
        <GroundTrack {traj} color={colorForPlane(parsePlaneFromId(id))} />
      {/each}
      {#each Array.from(comparatorTrajectoryById.entries()) as [id, traj] (id)}
        <GroundTrack {traj} color={COMPARATOR_COLOR} />
      {/each}
    </T.Group>

    {#each Array.from(trajectoryById.entries()) as [id, traj] (id)}
      <SatelliteMarker {traj} color={colorForPlane(parsePlaneFromId(id))} />
    {/each}
    {#each Array.from(comparatorTrajectoryById.entries()) as [id, traj] (id)}
      <SatelliteMarker {traj} color={COMPARATOR_COLOR} />
    {/each}
  </Canvas>
</div>
