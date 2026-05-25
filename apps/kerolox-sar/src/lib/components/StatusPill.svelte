<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  // Phase 1: always "idle". Phase 2 widens this to: idle | computing | done | error | cancelled.
  type Status = "idle";
  const status: Status = "idle";
</script>

<span
  class="inline-flex items-center gap-1 px-2 py-0.5 text-xs rounded-full border border-neutral-700 bg-neutral-900 text-neutral-300"
>
  <span class="size-2 rounded-full bg-neutral-500"></span>
  {status}
</span>
