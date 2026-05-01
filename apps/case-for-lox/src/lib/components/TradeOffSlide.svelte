<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide, Transition } from '@animotion/core';
</script>

<Slide class="h-full place-content-center place-items-center">
	<p class="text-5xl font-bold">Choose Two</p>

	<div class="mt-12 grid grid-cols-3 gap-6 text-center text-xl">
		<div class="rounded-xl border border-zinc-700 bg-zinc-900 p-6">
			<p class="text-2xl font-bold text-white">STK / GMAT</p>
			<p class="mt-3 text-green-400">✓ Interactive</p>
			<p class="text-green-400">✓ Performant</p>
			<p class="text-red-500">✗ Flexible</p>
		</div>
		<div class="rounded-xl border border-zinc-700 bg-zinc-900 p-6">
			<p class="text-2xl font-bold text-white">Orekit</p>
			<p class="mt-3 text-red-500">✗ Interactive</p>
			<p class="text-green-400">✓ Performant</p>
			<p class="text-green-400">✓ Flexible</p>
		</div>
		<div class="rounded-xl border border-zinc-700 bg-zinc-900 p-6">
			<p class="text-2xl font-bold text-white">Python / MATLAB</p>
			<p class="mt-3 text-green-400">✓ Interactive</p>
			<p class="text-red-500">✗ Performant</p>
			<p class="text-green-400">✓ Flexible</p>
		</div>
	</div>

	<Transition class="mt-12 text-3xl text-zinc-300">
		Can we have our cake and eat it too?
	</Transition>

	<Notes>
		Existing solutions all force us into a "choose two" situation due to their inherent trade-offs.
		STK and GMAT are interactive and performant enough but inflexible — you work the way they want
		you to work. Orekit is flexible and performant but not interactive — it's a Java library, not a
		live environment. Python and MATLAB are flexible and interactive but not performant enough for
		serious Monte Carlo work. This led me to ask the question whether we can have our cake and eat
		it too.
	</Notes>
</Slide>
