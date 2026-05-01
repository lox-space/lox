<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide } from '@animotion/core';
	import study from '$lib/assets/study.png';
</script>

<Slide class="h-full bg-black p-0">
	<div class="absolute inset-0 flex items-center justify-center">
		<img src={study} alt="Comparative study results" class="max-h-full max-w-full object-contain" />
	</div>

	<Notes>
		In 2016 I ran an experiment to test whether the performance gap between static and dynamic
		languages could be bridged by using techniques such as just-in-time compilation. The results
		were very encouraging and I got to work. After working with Python and the Numba JIT compiler
		for a while I switched to Julia and the rest is history. Well not quite..
	</Notes>
</Slide>
