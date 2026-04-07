<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
  import {
    Pane,
    ThemeUtils,
    Slider,
    Checkbox,
    Color,
  } from "svelte-tweakpane-ui";

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
</script>

<Pane
  theme={ThemeUtils.presets.light}
  position="fixed"
  title="Keplerian Elements"
>
  <Slider
    label="Semi-Major Axis [km]"
    bind:value={semiMajorAxis}
    min={7000}
    max={100000}
    step={100}
  />
  <Slider
    label="Eccentricity"
    bind:value={eccentricity}
    min={0.01}
    max={0.99}
    step={0.01}
  />
  <Slider
    label="Inclination [deg]"
    bind:value={inclination}
    min={0}
    max={180}
    step={1}
  />
  <Slider
    label="RAAN [deg]"
    bind:value={raan}
    min={0}
    max={360}
    step={1}
  />
  <Slider
    label="Arg. of Periapsis [deg]"
    bind:value={argPeriapsis}
    min={0}
    max={360}
    step={1}
  />
  <Slider
    label="True Anomaly [deg]"
    bind:value={trueAnomaly}
    min={-180}
    max={180}
    step={1}
  />
  <Color label="Orbit Color" bind:value={color} />
  <Checkbox label="Equatorial Plane" bind:value={withEquatorialPlane} />
  <Checkbox label="Orbital Plane" bind:value={withOrbitalPlane} />
</Pane>
