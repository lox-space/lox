<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
  import { T } from "@threlte/core";
  import { Gizmo, OrbitControls, SVG } from "@threlte/extras";
  import Earth from "../Earth.svelte";
  import KeplerianOrbit from "../KeplerianOrbit.svelte";
  import {
    DoubleSide,
    Euler,
    Group,
    Matrix4,
    Quaternion,
    type QuaternionTuple,
  } from "three";
  import { deg_to_rad } from "@lox-space/wasm";

  interface Props {
    semiMajorAxis: number;
    eccentricity: number;
    inclination: number;
    longitudeOfAscendingNode: number;
    argumentOfPeriapsis: number;
    trueAnomaly: number;
    withEquatorialPlane: boolean;
    withOrbitalPlane: boolean;
    color: string;
  }

  let {
    semiMajorAxis,
    eccentricity,
    inclination,
    longitudeOfAscendingNode,
    argumentOfPeriapsis,
    trueAnomaly,
    withEquatorialPlane,
    withOrbitalPlane,
    color,
  }: Props = $props();

  let planeRotation = $derived.by((): [number, number, number, number] => {
    let euler = new Euler(
      Math.PI / 2 + deg_to_rad(inclination),
      deg_to_rad(longitudeOfAscendingNode),
      deg_to_rad(-argumentOfPeriapsis),
      "YXZ"
    );
    let quat = new Quaternion().setFromEuler(euler);
    return [quat.x, quat.y, quat.z, quat.w];
  });
  let planeDimensions = $derived.by((): [number, number] => {
    const semiMinorAxis = semiMajorAxis * Math.sqrt(1 - eccentricity ** 2);
    return [semiMajorAxis * 2.1, semiMinorAxis * 2.1];
  });
  let linearEccentricity = $derived(eccentricity * semiMajorAxis);
</script>

<!-- Helper -->
<T.GridHelper args={[1e5, 1e1]} visible={withEquatorialPlane} />

<!-- Camera -->
<T.PerspectiveCamera makeDefault position={[0, 0, 7e4]} far={1e12}>
  <OrbitControls>
    <Gizmo />
  </OrbitControls>
</T.PerspectiveCamera>

<!-- Light -->
<T.AmbientLight intensity={2} />

<!-- Action -->
<Earth />
<KeplerianOrbit
  {semiMajorAxis}
  {eccentricity}
  {inclination}
  {longitudeOfAscendingNode}
  {argumentOfPeriapsis}
  {trueAnomaly}
  {color}
/>
<T.Group quaternion={planeRotation}>
  <T.Mesh position={[-linearEccentricity, 0, 0]} visible={withOrbitalPlane}>
    <T.PlaneGeometry args={planeDimensions} />
    <T.MeshBasicMaterial
      transparent={true}
      {color}
      opacity={0.3}
      side={DoubleSide}
    />
  </T.Mesh>
</T.Group>
