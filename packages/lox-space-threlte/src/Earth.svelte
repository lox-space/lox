<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
  import { Origin } from "@lox-space/wasm";
  import { T } from "@threlte/core";
  import { useTexture } from "@threlte/extras";

  interface Props {
    textureUrl?: string;
  }

  let { textureUrl }: Props = $props();

  const earth = new Origin("Earth");
  // WASM returns meters, Three.js scene uses km
  const meanRadius = earth.mean_radius() / 1000;
</script>

<T.Mesh>
  <T.SphereGeometry args={[meanRadius, 64, 64]} />
  {#if textureUrl}
    {#await useTexture(textureUrl) then texture}
      <T.MeshStandardMaterial map={texture} />
    {/await}
  {:else}
    <T.MeshStandardMaterial color="#4488ff" />
  {/if}
</T.Mesh>
