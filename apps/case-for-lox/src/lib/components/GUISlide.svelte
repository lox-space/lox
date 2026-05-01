<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide, Transition } from '@animotion/core';
</script>

<Slide class="h-full place-content-center place-items-center">
	<blockquote class="max-w-3xl text-center">
		<p class="text-3xl leading-relaxed text-zinc-300">
			"Okay, you showed us a bunch of code.<br />
			When can we have a
			<span class="font-bold text-white">graphical user interface</span>
			and some
			<span class="font-bold text-white">eye candy</span>?"
		</p>
		<footer class="mt-4 text-lg text-zinc-500">— Conference audience, repeatedly</footer>
	</blockquote>

	<Transition class="mt-16 text-center">
		<p class="text-6xl font-bold tabular-nums text-sky-400">~0</p>
		<p class="mt-2 text-xl text-zinc-400">meaningful outside contributions in 10 years</p>
	</Transition>

	<Notes>
		Finding users also proved to be nigh impossible. Whenever I paraded my Jupyter notebooks around
		at conferences the reaction from the audience was mostly blank stares and one or two questions
		that could be paraphrased as "okay, you showed us a bunch of code. When can we have a graphical
		user interface and some eye candy?"

		Even more painful was the fact that in almost 10 years I did not receive any meaningful outside
		contributions to my Julia projects except for a few bug fixes.
	</Notes>
</Slide>
