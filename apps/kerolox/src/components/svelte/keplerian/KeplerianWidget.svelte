<script lang="ts">
  import init, { rad_to_deg } from "@lox-space/wasm";
  import { Canvas } from "@threlte/core";
  import { WebGLRenderer } from "three";
  import Scene from "./Scene.svelte";
  import Settings from "./Settings.svelte";

  await init();

  let semiMajorAxis = $state(24464.0);
  let eccentricity = $state(0.7311);
  let inclination = $state(rad_to_deg(0.122138));
  let longitudeOfAscendingNode = $state(rad_to_deg(1.00681));
  let argumentOfPeriapsis = $state(rad_to_deg(3.10686));
  let trueAnomaly = $state(rad_to_deg(0.44369564302687126));
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
    <Scene
      {semiMajorAxis}
      {eccentricity}
      {inclination}
      {longitudeOfAscendingNode}
      {argumentOfPeriapsis}
      {trueAnomaly}
      {withEquatorialPlane}
      {withOrbitalPlane}
      {color}
    />
  </Canvas>
</div>

<Settings
  bind:semiMajorAxis
  bind:eccentricity
  bind:inclination
  bind:longitudeOfAscendingNode
  bind:argumentOfPeriapsis
  bind:trueAnomaly
  bind:withEquatorialPlane
  bind:withOrbitalPlane
  bind:color
/>
