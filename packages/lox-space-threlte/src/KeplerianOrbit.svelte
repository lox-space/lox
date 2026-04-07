<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
  import { Keplerian as KeplerianWasm, Origin } from "@lox-space/wasm";
  import { T, useThrelte, useTask } from "@threlte/core";
  import {
    Billboard,
    MeshLineGeometry,
    MeshLineMaterial,
    Text,
  } from "@threlte/extras";
  import { Group, Vector3 } from "three";

  const DEG_TO_RAD = Math.PI / 180;
  const M_TO_KM = 1e-3;

  interface Props {
    semiMajorAxis: number;
    eccentricity: number;
    inclination: number;
    raan: number;
    argPeriapsis: number;
    trueAnomaly: number;
    origin?: string;
    color?: string;
    name?: string;
  }

  let {
    semiMajorAxis,
    eccentricity,
    inclination,
    raan,
    argPeriapsis,
    trueAnomaly,
    origin: originName = "Earth",
    color = "#e92093",
    name,
  }: Props = $props();

  let wasmOrigin = $derived(new Origin(originName));

  let orbit = $derived(
    new KeplerianWasm(
      semiMajorAxis * 1000,
      eccentricity,
      inclination * DEG_TO_RAD,
      raan * DEG_TO_RAD,
      argPeriapsis * DEG_TO_RAD,
      trueAnomaly * DEG_TO_RAD,
      wasmOrigin,
    )
  );

  let position = $derived.by((): [number, number, number] => {
    const pos = orbit.to_cartesian().to_threejs();
    return [pos[0] * M_TO_KM, pos[1] * M_TO_KM, pos[2] * M_TO_KM];
  });

  let points = $derived.by(() => {
    const buffer = orbit.trace(360).to_threejs_buffer();
    const pts: Vector3[] = [];
    for (let i = 0; i < buffer.length; i += 3) {
      pts.push(
        new Vector3(
          buffer[i] * M_TO_KM,
          buffer[i + 1] * M_TO_KM,
          buffer[i + 2] * M_TO_KM,
        ),
      );
    }
    return pts;
  });

  const { camera } = useThrelte();
  let billboardRef = $state<Group>();
  let fontSize = $state<number>(500);
  let scale = $state<number>(1);

  useTask(() => {
    if (billboardRef) {
      const distance = billboardRef.position.distanceTo($camera.position);
      fontSize = distance * 0.02;
      scale = distance * 0.001;
    }
  });
</script>

<T.Mesh>
  <MeshLineGeometry {points} />
  <MeshLineMaterial {color} attenuate={false} width={2} />
</T.Mesh>
{#if name}
  <Billboard {position} bind:ref={billboardRef}>
    <Text text={name} {fontSize} {color} />
    <T.Mesh>
      <T.CircleGeometry args={[scale * 100, 32]} />
      <T.MeshBasicMaterial {color} />
    </T.Mesh>
  </Billboard>
{/if}
