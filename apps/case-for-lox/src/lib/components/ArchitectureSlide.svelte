<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide, Transition } from '@animotion/core';
</script>

<Slide class="h-full place-content-center place-items-center">
	<!--
		flex-col-reverse: DOM order is Core → Bindings → Frontend,
		but visual order is Frontend (top) → Bindings → Core (bottom).
		Transitions therefore reveal bottom-up: Bindings first, then Frontend.
	-->
	<div class="flex w-full max-w-4xl flex-col-reverse items-stretch gap-0 text-base">

		<!-- Layer 1: Rust Core — always visible, appears at bottom -->
		<div class="rounded-xl border border-orange-500/40 bg-orange-950/10 px-8 py-5">
			<p class="mb-4 text-sm font-bold uppercase tracking-widest" style="color: #f74c00;">Rust · 15 crates</p>
			<div class="flex justify-between gap-4 text-sm text-zinc-300">
				<div>
					<p class="mb-2 text-sm text-zinc-500">Time &amp; Data</p>
					<p class="font-mono text-zinc-400">lox-time</p>
					<p class="font-mono text-zinc-400">lox-bodies</p>
					<p class="font-mono text-zinc-400">lox-earth</p>
					<p class="font-mono text-zinc-400">lox-ephem</p>
				</div>
				<div class="border-l border-zinc-700 pl-6">
					<p class="mb-2 text-sm text-zinc-500">Math &amp; Core</p>
					<p class="font-mono text-zinc-400">lox-math</p>
					<p class="font-mono text-zinc-400">lox-units</p>
					<p class="font-mono text-zinc-400">lox-core</p>
					<p class="font-mono text-zinc-400">lox-frames</p>
				</div>
				<div class="border-l border-zinc-700 pl-6">
					<p class="mb-2 text-sm text-zinc-500">Domain</p>
					<p class="font-mono text-zinc-400">lox-orbits</p>
					<p class="font-mono text-zinc-400">lox-analysis</p>
					<p class="font-mono text-zinc-400">lox-comms</p>
					<p class="font-mono text-zinc-400">lox-itur · lox-io</p>
				</div>
			</div>
		</div>

		<!-- Layer 2: Bindings — Transition 1, appears in middle -->
		<Transition class="flex flex-col items-stretch">
			<div class="flex justify-center py-2">
				<div class="h-5 w-px bg-zinc-600"></div>
			</div>
			<div class="flex gap-4">
				<div class="flex-1 rounded-xl border border-sky-500/40 bg-sky-950/20 px-6 py-4">
					<p class="mb-1 text-sm font-bold uppercase tracking-widest text-sky-400">Python</p>
					<p class="font-mono text-zinc-300">lox-space</p>
					<p class="mt-1 text-sm text-zinc-500">PyO3 · maturin</p>
				</div>
				<div class="flex-1 rounded-xl border border-purple-500/40 bg-purple-950/20 px-6 py-4">
					<p class="mb-1 text-sm font-bold uppercase tracking-widest text-purple-400">WebAssembly</p>
					<p class="font-mono text-zinc-300">lox-space-wasm</p>
					<p class="mt-1 text-sm text-zinc-500">wasm-bindgen · wasm-pack</p>
				</div>
			</div>
		</Transition>

		<!-- Layer 3: Kerolox — Transition 2, appears second from top -->
		<Transition class="flex flex-col items-stretch">
			<div class="flex justify-center py-2">
				<div class="h-5 w-px bg-zinc-600"></div>
			</div>
			<div class="rounded-xl border border-green-500/40 bg-green-950/20 px-8 py-4">
				<p class="mb-3 text-sm font-bold uppercase tracking-widest text-green-400">Kerolox · Svelte packages</p>
				<div class="flex justify-center gap-6 font-mono text-zinc-300">
					<span>lox-space-svelte</span>
					<span class="text-zinc-600">·</span>
					<span>lox-space-threlte</span>
				</div>
			</div>
		</Transition>

		<!-- Layer 4: Consumers — Transition 3, appears at very top -->
		<Transition class="flex flex-col items-stretch">
			<div class="rounded-xl border border-zinc-600/40 bg-zinc-900/40 px-8 py-4">
				<p class="mb-3 text-sm font-bold uppercase tracking-widest text-zinc-500">Your application</p>
				<div class="flex justify-center gap-6 text-zinc-400">
					<span>web apps</span>
					<span class="text-zinc-600">·</span>
					<span>web services</span>
					<span class="text-zinc-600">·</span>
					<span>desktop apps</span>
					<span class="text-zinc-600">·</span>
					<span>notebooks</span>
					<span class="text-zinc-600">·</span>
					<span>scripts</span>
				</div>
			</div>
		</Transition>

	</div>

	<Notes>
		So what did we build? Lox is structured as a collection of focused Rust crates at the
		foundation. We have crates for time scale handling, celestial body data, Earth models and
		orientation, ephemeris access — and on top of those, the domain-level crates for orbital
		mechanics, mission analysis, telecommunications, and propagation models. The whole Rust layer is
		usable as a standalone library. On top of that we have two binding targets: lox-space exposes a
		Python API via PyO3 and maturin, so scientists and engineers can use Lox from Python the same
		way they would have used poliastro. And lox-space-wasm compiles the same core to WebAssembly,
		which is what powers all the interactive demos you saw today. Finally, a set of JavaScript
		packages wrap the WASM module with idiomatic Svelte, React, and Three.js components.
	</Notes>
</Slide>
