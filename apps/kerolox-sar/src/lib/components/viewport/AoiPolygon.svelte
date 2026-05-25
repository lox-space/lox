<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { T } from "@threlte/core";
  import { Origin } from "@lox-space/wasm";
  import {
    BufferGeometry,
    DoubleSide,
    Float32BufferAttribute,
    LineBasicMaterial,
    MeshBasicMaterial,
  } from "three";
  import type { AoiPolygon } from "$lib/aois";

  // A warm pink-red contrasts strongly against the predominantly blue/green/tan
  // day side AND the dark night side (cyan blended into the lit ocean); amber
  // stays reserved for the ICEYE comparator fleet. The interior is a translucent
  // fill so the AOI reads as an area, not a thin line.
  let {
    aoi,
    color = "#ff3366",
    fillOpacity = 0.35,
  }: { aoi: AoiPolygon; color?: string; fillOpacity?: number } = $props();

  const earth = new Origin("Earth");
  const earthRadiusKm = earth.mean_radius() / 1000;
  $effect(() => () => { earth.free(); });

  // Fill sits just above the surface; the outline sits a hair higher so it
  // always draws cleanly on top of its own fill.
  const fillRadius = earthRadiusKm * 1.002;
  const lineRadius = earthRadiusKm * 1.003;
  // Sub-segments per edge: straight 3D chords between distant vertices would
  // sink below the sphere (clipping behind the globe), so densify each edge in
  // lon/lat and re-project onto the sphere to make the polygon hug the surface.
  const SEG_PER_EDGE = 16;

  function lonLatToVec3(lon: number, lat: number, r: number): [number, number, number] {
    const phi = (lat * Math.PI) / 180;
    const lam = (lon * Math.PI) / 180;
    const cl = Math.cos(phi);
    return [r * cl * Math.cos(lam), r * Math.sin(phi), -r * cl * Math.sin(lam)];
  }

  // Densified boundary in lon/lat (each edge subdivided, ring left open).
  const boundary = $derived.by((): [number, number][] => {
    const verts = aoi.exteriorLonLat;
    const n = verts.length;
    const out: [number, number][] = [];
    if (n < 2) return out;
    for (let i = 0; i < n; i++) {
      const [lon0, lat0] = verts[i];
      const [lon1, lat1] = verts[(i + 1) % n];
      for (let s = 0; s < SEG_PER_EDGE; s++) {
        const t = s / SEG_PER_EDGE;
        out.push([lon0 + (lon1 - lon0) * t, lat0 + (lat1 - lat0) * t]);
      }
    }
    return out;
  });

  // Closed outline ring on the sphere.
  const lineGeometry = $derived.by(() => {
    const out: number[] = [];
    for (const [lon, lat] of boundary) {
      const [x, y, z] = lonLatToVec3(lon, lat, lineRadius);
      out.push(x, y, z);
    }
    if (boundary.length) {
      const [x, y, z] = lonLatToVec3(boundary[0][0], boundary[0][1], lineRadius);
      out.push(x, y, z);
    }
    const g = new BufferGeometry();
    g.setAttribute("position", new Float32BufferAttribute(new Float32Array(out), 3));
    return g;
  });

  // Translucent cap: triangle fan from the AOI centroid over the boundary.
  const fillGeometry = $derived.by(() => {
    const b = boundary;
    const g = new BufferGeometry();
    if (b.length < 3) return g;
    const verts = aoi.exteriorLonLat;
    let clon = 0;
    let clat = 0;
    for (const [lon, lat] of verts) {
      clon += lon;
      clat += lat;
    }
    const c = lonLatToVec3(clon / verts.length, clat / verts.length, fillRadius);
    const out: number[] = [];
    for (let i = 0; i < b.length; i++) {
      const a = lonLatToVec3(b[i][0], b[i][1], fillRadius);
      const d = lonLatToVec3(b[(i + 1) % b.length][0], b[(i + 1) % b.length][1], fillRadius);
      out.push(c[0], c[1], c[2], a[0], a[1], a[2], d[0], d[1], d[2]);
    }
    g.setAttribute("position", new Float32BufferAttribute(new Float32Array(out), 3));
    return g;
  });

  const fillMaterial = $derived(
    new MeshBasicMaterial({
      color,
      transparent: true,
      opacity: fillOpacity,
      side: DoubleSide,
      depthWrite: false,
    }),
  );
  const lineMaterial = $derived(new LineBasicMaterial({ color }));
</script>

<T.Mesh args={[fillGeometry, fillMaterial]} />
<T.Line args={[lineGeometry, lineMaterial]} />
