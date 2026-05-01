<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide, Transition } from '@animotion/core';
</script>

<Slide class="h-full place-content-center place-items-center">
	<!-- The gap -->
	<div class="flex w-full max-w-4xl items-center justify-between gap-4 text-center">
		<div class="w-44 rounded-xl border border-zinc-700 bg-zinc-900 px-6 py-5">
			<p class="text-lg font-bold text-zinc-300">Low-level<br />library</p>
			<p class="mt-2 text-sm text-zinc-500">Orekit · poliastro</p>
		</div>

		<Transition class="flex flex-1 flex-col items-center gap-2">
			<p class="text-sm font-bold tracking-widest text-zinc-500 uppercase">The missing link</p>
			<div class="flex w-full items-center">
				<div class="h-px flex-1 bg-zinc-600"></div>
				<div class="mx-3 rounded-xl border border-sky-500/50 bg-sky-950/30 px-5 py-4 text-center">
					<p class="text-lg font-bold text-white">Framework</p>
					<p class="mt-1 text-sm text-zinc-400">
						algorithms <span class="text-zinc-600">+</span> widgets
					</p>
				</div>
				<div class="h-px flex-1 bg-zinc-600"></div>
			</div>
		</Transition>

		<div class="w-44 rounded-xl border border-zinc-700 bg-zinc-900 px-6 py-5">
			<p class="text-lg font-bold text-zinc-300">GUI<br />application</p>
			<p class="mt-2 text-sm text-zinc-500">STK · GMAT</p>
		</div>
	</div>

	<!-- The principle -->
	<Transition class="mt-12 w-full max-w-4xl">
		<div class="flex items-center justify-center gap-6 text-xl">
			<div
				class="rounded-lg border border-orange-500/40 bg-orange-950/10 px-6 py-4 font-mono text-zinc-300"
			>
				<span class="text-zinc-500">struct</span>
				<span style="color: #f74c00;">KeplerianElements</span>
			</div>
			<span class="text-2xl text-zinc-500">↔</span>
			<div
				class="rounded-lg border border-green-500/40 bg-green-950/10 px-6 py-4 font-mono text-zinc-300"
			>
				<span class="text-zinc-500">&lt;</span><span class="text-green-400">KeplerianSettings</span>
				<span class="text-zinc-500">/&gt;</span>
			</div>
		</div>
		<p class="mt-4 text-center text-zinc-500">every algorithm has a corresponding widget</p>
	</Transition>
	<Notes>
		I want to zoom out for a moment and explain the bigger vision. If you look at the mission
		analysis software landscape, there are two extremes. On one end you have GUI applications like
		STK and GMAT — feature-rich and discoverable but expensive, inflexible, and hard to integrate
		into automated workflows. On the other end you have low-level libraries like Orekit or poliastro
		— powerful and flexible but requiring significant expertise and custom development to build
		anything useful on top of them. The missing link in the middle is a framework that provides
		reusable building blocks at both layers simultaneously: the algorithmic layer and the UI layer.
		The guiding principle is simple: every Rust struct in Lox should have a corresponding Svelte
		widget in Kerolox. You saw this in action with the Keplerian demo — KeplerianElements on the
		Rust side, KeplerianSettings on the Svelte side. The same data model, two representations. If we
		do this consistently across the whole library, we get something powerful: the same components
		can be assembled into Python scripts for exploratory analysis, into full GUI applications like
		Replica Horizon, or eventually into a low-code environment where non-experts can build their own
		mission analysis workflows by wiring components together visually.
	</Notes>
</Slide>
