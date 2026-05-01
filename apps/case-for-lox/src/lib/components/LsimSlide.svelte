<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide, Transition } from '@animotion/core';
</script>

<Slide class="h-full place-content-center place-items-center">
	<p class="mb-12 text-4xl font-bold">LSIM — Lunar Service Volume Simulator</p>

	<div class="flex items-center gap-0 text-center text-lg">
		<!-- React box -->
		<div class="rounded-xl border border-zinc-700 bg-zinc-900 px-6 py-5 w-44">
			<p class="text-2xl font-bold text-sky-400">React</p>
			<p class="mt-1 text-zinc-400">Frontend</p>
		</div>

		<!-- Arrow 1 -->
		<div class="flex flex-col items-center">
			<Transition class="mb-1 rounded bg-zinc-800 px-3 py-1 text-sm text-red-400">
				math? reimpl. or round-trip
			</Transition>
			<div class="flex items-center">
				<div class="h-px w-16 bg-zinc-500"></div>
				<span class="text-zinc-500">▶</span>
			</div>
		</div>

		<!-- TypeScript box -->
		<div class="rounded-xl border border-zinc-700 bg-zinc-900 px-6 py-5 w-44">
			<p class="text-2xl font-bold text-sky-400">TypeScript</p>
			<p class="mt-1 text-zinc-400">Backend</p>
		</div>

		<!-- Arrow 2 -->
		<div class="flex items-center">
			<div class="h-px w-16 bg-zinc-500"></div>
			<span class="text-zinc-500">▶</span>
		</div>

		<!-- Julia box -->
		<div class="relative rounded-xl border border-zinc-700 bg-zinc-900 px-6 py-5 w-44">
			<p class="text-2xl font-bold text-sky-400">Julia</p>
			<p class="mt-1 text-zinc-400">Workers</p>
			<Transition class="absolute -bottom-10 left-1/2 -translate-x-1/2 whitespace-nowrap rounded bg-zinc-800 px-3 py-1 text-sm text-red-400">
				JIT · slow test compilation
			</Transition>
		</div>
	</div>

	<Notes>
		In 2021, I built a GUI for LSIM, the Lunar Service Volume Simulator, for ESA's Project
		Moonlight while at Telespazio. Doing so, I finally hit the wall with my Julia projects. LSIM
		was a web service with a React frontend and a backend written in TypeScript that talked to
		several simulation workers running my Julia astrodynamics code. Every time there was math to be
		done in the UI we had to either hit the network or re-implement it in TypeScript, forcing us to
		choose between bad user experience or bad developer experience. And we learned that JIT
		compilation is not a free lunch, especially for testing. Julia did not support incremental
		compilation of our test suite and as a result it got slower and slower over time because we paid
		the compilation cost each time we ran it.
	</Notes>
</Slide>
