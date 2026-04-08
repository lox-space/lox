<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
  import init from "@lox-space/wasm";
  import { T, Canvas } from "@threlte/core";
  import { Gizmo, OrbitControls } from "@threlte/extras";
  import { WebGLRenderer } from "three";
  import { Earth, KeplerianOrbit } from "@lox-space/threlte";
  import { KeplerianSettings } from "@lox-space/svelte";

  await init();

  let semiMajorAxis = $state(24464);
  let eccentricity = $state(0.7311);
  let inclination = $state(7.0);
  let raan = $state(57.7);
  let argPeriapsis = $state(178.1);
  let trueAnomaly = $state(25.4);
  let withEquatorialPlane = $state(false);
  let withOrbitalPlane = $state(false);
  let color = $state("#e92093");
</script>

<div class="fixed inset-0 overflow-hidden">
  <Canvas
    createRenderer={(canvas) => {
      return new WebGLRenderer({
        canvas,
        logarithmicDepthBuffer: true,
      });
    }}
  >
    <T.GridHelper args={[1e5, 1e1]} visible={withEquatorialPlane} />

    <T.PerspectiveCamera makeDefault position={[0, 0, 7e4]} far={1e12}>
      <OrbitControls>
        <Gizmo xColor="#ff4060" yColor="#40ff60" zColor="#4060ff" labelX="X" labelY="Z" labelZ="-Y" />
      </OrbitControls>
    </T.PerspectiveCamera>

    <T.AmbientLight intensity={2} />

    <Earth textureUrl="/assets/Earth-color.jpg" />
    <KeplerianOrbit
      {semiMajorAxis}
      {eccentricity}
      {inclination}
      {raan}
      {argPeriapsis}
      {trueAnomaly}
      {color}
      name="Sat1"
    />
  </Canvas>
</div>

<KeplerianSettings
  bind:semiMajorAxis
  bind:eccentricity
  bind:inclination
  bind:raan
  bind:argPeriapsis
  bind:trueAnomaly
  bind:color
  bind:withEquatorialPlane
  bind:withOrbitalPlane
/>
