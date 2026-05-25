<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { T } from "@threlte/core";
  import { Origin } from "@lox-space/wasm";
  import { BufferGeometry, Float32BufferAttribute, LineBasicMaterial } from "three";
  import type { AoiPolygon } from "$lib/aois";

  // Bright cyan reads clearly against both the sunlit and night sides of the
  // globe; amber stays reserved for the ICEYE comparator fleet.
  let { aoi, color = "#33e1ff" }: { aoi: AoiPolygon; color?: string } = $props();

  const earth = new Origin("Earth");
  const earthRadiusKm = earth.mean_radius() / 1000;
  $effect(() => () => { earth.free(); });

  // Lift the outline ~20 km above the surface to avoid z-fighting with the
  // textured sphere.
  const liftRadius = earthRadiusKm * 1.003;
  // Sub-segments per edge: straight 3D chords between distant vertices would
  // sink below the sphere (clipping behind the globe), so densify each edge in
  // lon/lat and re-project onto the sphere to make the outline hug the surface.
  const SEG_PER_EDGE = 16;

  function lonLatToVec3(lon: number, lat: number, r: number): [number, number, number] {
    const phi = (lat * Math.PI) / 180;
    const lam = (lon * Math.PI) / 180;
    const cl = Math.cos(phi);
    return [r * cl * Math.cos(lam), r * Math.sin(phi), -r * cl * Math.sin(lam)];
  }

  const points = $derived.by((): Float32Array => {
    const verts = aoi.exteriorLonLat;
    const n = verts.length;
    if (n < 2) return new Float32Array(0);
    const out: number[] = [];
    // Walk every edge (wrapping past the last vertex to close the ring) and
    // emit SEG_PER_EDGE interpolated, sphere-projected points along it.
    for (let i = 0; i < n; i++) {
      const [lon0, lat0] = verts[i];
      const [lon1, lat1] = verts[(i + 1) % n];
      for (let s = 0; s < SEG_PER_EDGE; s++) {
        const t = s / SEG_PER_EDGE;
        const [x, y, z] = lonLatToVec3(lon0 + (lon1 - lon0) * t, lat0 + (lat1 - lat0) * t, liftRadius);
        out.push(x, y, z);
      }
    }
    // Close the ring back onto the first vertex.
    const [x, y, z] = lonLatToVec3(verts[0][0], verts[0][1], liftRadius);
    out.push(x, y, z);
    return new Float32Array(out);
  });

  const geometry = $derived.by(() => {
    const g = new BufferGeometry();
    g.setAttribute("position", new Float32BufferAttribute(points, 3));
    return g;
  });

  const material = $derived(new LineBasicMaterial({ color, linewidth: 2 }));
</script>

<T.Line args={[geometry, material]} />
