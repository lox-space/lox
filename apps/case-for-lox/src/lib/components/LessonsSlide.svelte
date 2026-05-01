<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide, Transition } from '@animotion/core';
</script>

<Slide class="h-full place-content-center place-items-center">
	<p class="mb-10 text-5xl font-bold">Lessons Learned</p>

	<ul class="space-y-6 text-left text-2xl text-zinc-300">
		<Transition as="li">
			<span class="font-bold text-white">Top-down optimisation is a dead end</span>
			<span class="ml-3 text-zinc-400">— fast foundations are a must</span>
		</Transition>
		<Transition as="li">
			<span class="font-bold text-white">You don't have to choose</span>
			<span class="ml-3 text-zinc-400">— Rust + PyO3 + WASM covers all bases</span>
		</Transition>
		<Transition as="li">
			<span class="font-bold text-white">Graphics are not just nice-to-have</span>
			<span class="ml-3 text-zinc-400">— they unlock a lot of potential</span>
		</Transition>
		<Transition as="li">
			<span class="font-bold text-white">Rust is (not) perfect</span>
			<span class="ml-3 text-zinc-400">— do(n't) panic, unwrap and clone all the things</span>
		</Transition>
	</ul>
</Slide>
