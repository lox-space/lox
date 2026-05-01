<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide } from '@animotion/core';
	import poliastroDead from '$lib/assets/poliastro-dead.png';
</script>

<Slide class="h-full place-content-center place-items-center">
	<img src={poliastroDead} alt="poliastro archived" class="max-h-[80vh] max-w-full" />

	<Notes>
		In short, poliastro's creator rage-quit the space industry, poliastro's current maintainer was
		barred from working on it and open source in general by his employer due to a perceived conflict
		of interest, and our team suddenly missed two key people and a main dependency. We decided that
		taking over maintainership or forking was not an option. After looking into alternatives we ended
		up with this choice.
	</Notes>
</Slide>
