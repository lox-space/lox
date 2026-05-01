<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
  interface Props {
    semiMajorAxis: number;
    eccentricity: number;
    inclination: number;
    raan: number;
    argPeriapsis: number;
    trueAnomaly: number;
    color: string;
    withEquatorialPlane: boolean;
    withOrbitalPlane: boolean;
  }

  let {
    semiMajorAxis = $bindable(),
    eccentricity = $bindable(),
    inclination = $bindable(),
    raan = $bindable(),
    argPeriapsis = $bindable(),
    trueAnomaly = $bindable(),
    color = $bindable(),
    withEquatorialPlane = $bindable(),
    withOrbitalPlane = $bindable(),
  }: Props = $props();

  const PERIGEE_MIN_KM = 6571; // Earth radius + 200 km

  let minSma = $derived(Math.ceil(PERIGEE_MIN_KM / (1 - eccentricity) / 100) * 100);

  $effect(() => {
    if (semiMajorAxis < minSma) semiMajorAxis = minSma;
  });
</script>

<div class="lox-panel">
  <p class="lox-panel-title">Keplerian Elements</p>

  <div class="lox-rows">
    <label class="lox-row">
      <span class="lox-label">Semi-Major Axis</span>
      <span class="lox-unit">km</span>
      <input type="range" min={minSma} max={100000} step={100} bind:value={semiMajorAxis} />
      <span class="lox-value">{semiMajorAxis.toFixed(0)}</span>
    </label>

    <label class="lox-row">
      <span class="lox-label">Eccentricity</span>
      <span class="lox-unit"></span>
      <input type="range" min={0.01} max={0.99} step={0.01} bind:value={eccentricity} />
      <span class="lox-value">{eccentricity.toFixed(2)}</span>
    </label>

    <label class="lox-row">
      <span class="lox-label">Inclination</span>
      <span class="lox-unit">deg</span>
      <input type="range" min={0} max={180} step={1} bind:value={inclination} />
      <span class="lox-value">{inclination.toFixed(0)}</span>
    </label>

    <label class="lox-row">
      <span class="lox-label">RAAN</span>
      <span class="lox-unit">deg</span>
      <input type="range" min={0} max={360} step={1} bind:value={raan} />
      <span class="lox-value">{raan.toFixed(0)}</span>
    </label>

    <label class="lox-row">
      <span class="lox-label">Arg. of Periapsis</span>
      <span class="lox-unit">deg</span>
      <input type="range" min={0} max={360} step={1} bind:value={argPeriapsis} />
      <span class="lox-value">{argPeriapsis.toFixed(0)}</span>
    </label>

    <label class="lox-row">
      <span class="lox-label">True Anomaly</span>
      <span class="lox-unit">deg</span>
      <input type="range" min={-180} max={180} step={1} bind:value={trueAnomaly} />
      <span class="lox-value">{trueAnomaly.toFixed(0)}</span>
    </label>

    <label class="lox-row">
      <span class="lox-label">Orbit Color</span>
      <span class="lox-unit"></span>
      <input type="color" bind:value={color} class="lox-color" />
      <span class="lox-value">{color}</span>
    </label>

    <label class="lox-row lox-checkbox-row">
      <input type="checkbox" bind:checked={withEquatorialPlane} />
      <span class="lox-label">Equatorial Plane</span>
    </label>

    <label class="lox-row lox-checkbox-row">
      <input type="checkbox" bind:checked={withOrbitalPlane} />
      <span class="lox-label">Orbital Plane</span>
    </label>
  </div>
</div>

<style>
  .lox-panel {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 100;
    background: rgba(15, 15, 20, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 0.5rem;
    padding: 0.75rem 1rem 1rem;
    width: 22rem;
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    color: #e4e4e7;
    backdrop-filter: blur(8px);
  }

  .lox-panel-title {
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #a1a1aa;
    margin-bottom: 0.75rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .lox-rows {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .lox-row {
    display: grid;
    grid-template-columns: 9rem 2rem 1fr 3.5rem;
    align-items: center;
    gap: 0.25rem;
    cursor: pointer;
  }

  .lox-label {
    color: #a1a1aa;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .lox-unit {
    color: #71717a;
    font-size: 0.65rem;
    text-align: right;
  }

  .lox-value {
    text-align: right;
    color: #f4f4f5;
    font-variant-numeric: tabular-nums;
  }

  input[type='range'] {
    width: 100%;
    accent-color: #38bdf8;
    cursor: pointer;
    height: 2px;
  }

  .lox-color {
    width: 2rem;
    height: 1.25rem;
    border: none;
    border-radius: 0.2rem;
    cursor: pointer;
    background: none;
    padding: 0;
  }

  .lox-checkbox-row {
    grid-template-columns: auto 1fr;
    gap: 0.5rem;
  }

  .lox-checkbox-row input[type='checkbox'] {
    accent-color: #38bdf8;
    width: 0.875rem;
    height: 0.875rem;
    cursor: pointer;
  }
</style>
