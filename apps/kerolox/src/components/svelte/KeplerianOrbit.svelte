<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
  import { KeplerianElements } from "@lox-space/wasm";
  import { T, useTask, useThrelte } from "@threlte/core";
  import {
    Billboard,
    HTML,
    MeshLineGeometry,
    MeshLineMaterial,
    SVG,
    Text,
  } from "@threlte/extras";
  import { Box2, Box3, Group, Vector3 } from "three";
  import { ICRF_TO_THREE } from "../../utils";

  const gravParam = 398600.43550702266;

  interface Props {
    semiMajorAxis: number;
    eccentricity: number;
    inclination: number;
    longitudeOfAscendingNode: number;
    argumentOfPeriapsis: number;
    trueAnomaly: number;
    color?: string;
    name?: string;
  }

  let {
    semiMajorAxis,
    eccentricity,
    inclination,
    longitudeOfAscendingNode,
    argumentOfPeriapsis,
    trueAnomaly,
    color,
    name,
  }: Props = $props();

  let orbit = $derived(
    new KeplerianElements(
      semiMajorAxis,
      eccentricity,
      inclination,
      longitudeOfAscendingNode,
      argumentOfPeriapsis,
      trueAnomaly
    )
  );

  let position = $derived.by((): [number, number, number] => {
    let [x, y, z] = orbit.position(gravParam);

    let v = new Vector3(x, y, z);
    v.applyMatrix4(ICRF_TO_THREE);

    return [v.x, v.y, v.z];
  });

  let points = $derived.by(() => {
    const points = orbit.trace(gravParam, 360);
    return Array.from(points.x).map((x, idx) => {
      let y = points.y[idx];
      let z = points.z[idx];

      let v = new Vector3(x, y, z);
      v.applyMatrix4(ICRF_TO_THREE);

      return v;
    });
  });

  const { camera } = useThrelte();
  let billboardRef = $state<Group>();
  let fontSize = $state<number>();
  let scale = $state<number>();

  useTask(() => {
    if (billboardRef) {
      const distance = billboardRef.position.distanceTo($camera.position);
      fontSize = distance * 0.02;
      scale = distance * 0.001;
      const bbox = new Box3().setFromObject(billboardRef);
      const size = new Vector3();
      bbox.getSize(size);
      console.log(size);
    }
    if (svgRef) {
      const bbox = new Box3().setFromObject(svgRef);
      const size = new Vector3();
      bbox.getSize(size);
      svgRef.position.set(-size.x / 2, size.y / 2, -size.z / 2);
    }
  });

  let svgRef = $state<Group>();
</script>

<T.Mesh>
  <MeshLineGeometry {points} />
  <MeshLineMaterial {color} attenuate={false} width={2} />
</T.Mesh>
<!-- <T.Mesh {position}>
  <T.SphereGeometry args={[200, 64, 64]} />
  <T.MeshStandardMaterial {color} />
</T.Mesh> -->
<Billboard {position} bind:ref={billboardRef}>
  <!-- <SVG
    src="../assets/satellite.svg"
    {scale}
    strokeMaterialProps={{ color: color }}
    bind:ref={svgRef}
  /> -->
  <!-- <HTML><span style:color style:fontSize={24}>Sat</span></HTML> -->
  <Text text={name ?? "Sat1"} {fontSize} {color} />
  <T.Mesh>
    <T.CircleGeometry args={[100, 32]} />
    <T.MeshBasicMaterial {color} />
  </T.Mesh>
</Billboard>
