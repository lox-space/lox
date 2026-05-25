<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { trajectoryById, comparatorTrajectoryById, type SampledTrajectoryView } from "$lib/state/trajectories.svelte";
  import { playback, tick } from "$lib/state/playback.svelte";
  import type { AoiPolygon as AoiPolygonData } from "$lib/aois";
  import { colorForPlane, parsePlaneFromId } from "./colors";

  /** Fixed amber for the fielded ICEYE comparator fleet. */
  const COMPARATOR_COLOR = "#ffaa44";

  let { aois }: { aois: Map<string, AoiPolygonData> } = $props();

  let canvas: HTMLCanvasElement;
  let basemap: HTMLImageElement | null = null;
  let lastFrame = performance.now();
  let raf = 0;

  interface MapRect { x: number; y: number; w: number; h: number; }

  /**
   * The largest 2:1 (360° lon × 180° lat) rectangle that fits in the canvas,
   * centered. Equirectangular maps must keep this aspect ratio or the Earth
   * stretches; the surrounding letterbox bars stay dark.
   */
  function mapRect(cw: number, ch: number): MapRect {
    let w = cw;
    let h = cw / 2;
    if (h > ch) {
      h = ch;
      w = ch * 2;
    }
    return { x: (cw - w) / 2, y: (ch - h) / 2, w, h };
  }

  function lonLatToXY(lon: number, lat: number, r: MapRect): [number, number] {
    return [r.x + ((lon + 180) / 360) * r.w, r.y + ((90 - lat) / 180) * r.h];
  }

  function draw(): void {
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const { width, height } = canvas;
    const rect = mapRect(width, height);

    // Dark backdrop fills the whole canvas (including letterbox bars).
    ctx.fillStyle = "#0a0a0a";
    ctx.fillRect(0, 0, width, height);
    if (basemap) {
      ctx.drawImage(basemap, rect.x, rect.y, rect.w, rect.h);
    } else {
      ctx.fillStyle = "#1a1a2e";
      ctx.fillRect(rect.x, rect.y, rect.w, rect.h);
    }

    // AOIs: translucent cyan fill + outline (amber is reserved for ICEYE).
    ctx.lineWidth = 2;
    ctx.strokeStyle = "#33e1ff";
    ctx.fillStyle = "rgba(51, 225, 255, 0.2)";
    for (const aoi of aois.values()) {
      ctx.beginPath();
      for (let i = 0; i < aoi.exteriorLonLat.length; i++) {
        const [lon, lat] = aoi.exteriorLonLat[i];
        const [x, y] = lonLatToXY(lon, lat, rect);
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
    }

    // Ground tracks (split at ±180 longitude crossings). Reduced opacity so
    // dozens of overlapping tracks don't obscure the world map. Current-
    // position dots are drawn at full opacity below for contrast. User tracks
    // are colored per plane; comparator (ICEYE) tracks are a fixed amber.
    ctx.save();
    ctx.globalAlpha = 0.6;
    ctx.lineWidth = 2;
    drawTracks(ctx, trajectoryById, (id) => colorForPlane(parsePlaneFromId(id)), rect);
    drawTracks(ctx, comparatorTrajectoryById, () => COMPARATOR_COLOR, rect);
    ctx.restore();

    // Current satellite positions: interpolate from ground-track samples at currentTime.
    const t = playback.currentTime;
    drawDots(ctx, trajectoryById, (id) => colorForPlane(parsePlaneFromId(id)), t, rect);
    drawDots(ctx, comparatorTrajectoryById, () => COMPARATOR_COLOR, t, rect);
  }

  function drawTracks(
    ctx: CanvasRenderingContext2D,
    trajectories: Map<string, SampledTrajectoryView>,
    colorOf: (id: string) => string,
    rect: MapRect,
  ): void {
    for (const [id, traj] of trajectories.entries()) {
      ctx.strokeStyle = colorOf(id);
      ctx.beginPath();
      let prevLon: number | null = null;
      const n = traj.groundDeg.length / 2;
      for (let i = 0; i < n; i++) {
        const lat = traj.groundDeg[2 * i];
        const lon = traj.groundDeg[2 * i + 1];
        const [x, y] = lonLatToXY(lon, lat, rect);
        if (prevLon !== null && Math.abs(lon - prevLon) > 180) {
          ctx.moveTo(x, y);
        } else if (i === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
        prevLon = lon;
      }
      ctx.stroke();
    }
  }

  function drawDots(
    ctx: CanvasRenderingContext2D,
    trajectories: Map<string, SampledTrajectoryView>,
    colorOf: (id: string) => string,
    t: number,
    rect: MapRect,
  ): void {
    for (const [id, traj] of trajectories.entries()) {
      const epochs = traj.epochsMs;
      if (epochs.length === 0) continue;
      let lo = 0;
      let hi = epochs.length - 1;
      while (hi - lo > 1) {
        const mid = (lo + hi) >> 1;
        if (epochs[mid] <= t) lo = mid;
        else hi = mid;
      }
      const t0 = epochs[lo];
      const t1 = epochs[hi];
      const f = t1 === t0 ? 0 : Math.max(0, Math.min(1, (t - t0) / (t1 - t0)));
      const lat = traj.groundDeg[2 * lo] + (traj.groundDeg[2 * hi] - traj.groundDeg[2 * lo]) * f;
      const lon = traj.groundDeg[2 * lo + 1] + (traj.groundDeg[2 * hi + 1] - traj.groundDeg[2 * lo + 1]) * f;
      const [x, y] = lonLatToXY(lon, lat, rect);
      ctx.fillStyle = colorOf(id);
      ctx.beginPath();
      ctx.arc(x, y, 5, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  function frame(now: number): void {
    const dt = (now - lastFrame) / 1000;
    lastFrame = now;
    tick(dt * 1000); // ms units to match playback bounds
    draw();
    raf = requestAnimationFrame(frame);
  }

  onMount(() => {
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;

    const ro = new ResizeObserver(() => {
      if (canvas.clientWidth > 0 && canvas.clientHeight > 0) {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
        draw();
      }
    });
    ro.observe(canvas);

    const img = new Image();
    img.onload = () => {
      basemap = img;
      draw();
    };
    img.src = "/assets/Earth-color.jpg";
    lastFrame = performance.now();
    raf = requestAnimationFrame(frame);
    return () => {
      ro.disconnect();
      cancelAnimationFrame(raf);
    };
  });
</script>

<div class="flex-1 min-h-0 relative">
  <canvas
    bind:this={canvas}
    class="w-full h-full block bg-neutral-950"
    data-testid="map-canvas"
  ></canvas>
</div>
