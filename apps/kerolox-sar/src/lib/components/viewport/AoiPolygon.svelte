<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { T } from "@threlte/core";
  import { Origin } from "@lox-space/wasm";
  import { BufferGeometry, Float32BufferAttribute, LineBasicMaterial } from "three";
  import type { AoiPolygon } from "$lib/aois";

  let { aoi, color = "#ffaa44" }: { aoi: AoiPolygon; color?: string } = $props();

  const earth = new Origin("Earth");
  const earthRadiusKm = earth.mean_radius() / 1000;
  $effect(() => () => { earth.free(); });

  const liftRadius = earthRadiusKm + 2;

  function lonLatToVec3(lon: number, lat: number, r: number): [number, number, number] {
    const phi = (lat * Math.PI) / 180;
    const lam = (lon * Math.PI) / 180;
    const cl = Math.cos(phi);
    return [r * cl * Math.cos(lam), r * Math.sin(phi), -r * cl * Math.sin(lam)];
  }

  const points = $derived.by((): Float32Array => {
    const n = aoi.exteriorLonLat.length;
    const out = new Float32Array(n * 3);
    for (let i = 0; i < n; i++) {
      const [lon, lat] = aoi.exteriorLonLat[i];
      const [x, y, z] = lonLatToVec3(lon, lat, liftRadius);
      out[3 * i] = x;
      out[3 * i + 1] = y;
      out[3 * i + 2] = z;
    }
    return out;
  });

  const geometry = $derived.by(() => {
    const g = new BufferGeometry();
    g.setAttribute("position", new Float32BufferAttribute(points, 3));
    return g;
  });

  const material = $derived(new LineBasicMaterial({ color, linewidth: 2 }));
</script>

<T.Line args={[geometry, material]} />
