<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { useTask } from "@threlte/core";
  import { tick } from "$lib/state/playback.svelte";

  // Must be a descendant of <Canvas> to access the Threlte scheduler.
  // GlobeView mounts <TickAdvancer /> inside its <Canvas> so playback
  // advances on the GL render loop while the Globe view is active.
  useTask((delta) => {
    tick(delta * 1000);
  });
</script>
