<script lang="ts">
  import { T } from "@threlte/core";
  import { useTexture } from "@threlte/extras";
  const meanRadius = 6371;
</script>

<T.Mesh>
  <T.SphereGeometry args={[meanRadius, 64, 64]} />
  {#await useTexture("/assets/Earth-color.jpg") then texture}
    <T.MeshStandardMaterial map={texture} />
  {/await}
</T.Mesh>
