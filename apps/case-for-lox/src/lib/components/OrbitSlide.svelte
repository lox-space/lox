<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Slide, Transition, Notes, Code } from '@animotion/core';
	import initWasm, { Keplerian, Origin } from '@lox-space/wasm';

	await initWasm();

	let altitude = $state(400);

	let rustCode: Code;
	let pythonCode: Code;
	let jsCode: Code;

	let period = $derived.by(() => {
		const earth = new Origin('Earth');
		const orbit = Keplerian.circular_from_altitude(altitude * 1000, earth);
		const p = orbit.orbital_period();
		orbit.free();
		earth.free();
		return p;
	});

	function rustSnippet(alt: number): string {
		return (
			`let orbit = CircularBuilder::new()\n` +
			`  .with_altitude(${alt.toFixed(1)}.km())\n` +
			`  .build()?;\n` +
			`orbit.orbital_period()`
		);
	}

	function pythonSnippet(alt: number): string {
		return (
			`orbit = lox.Keplerian.circular(\n` +
			`    altitude=${Math.round(alt)} * lox.km\n` +
			`)\n` +
			`orbit.orbital_period()`
		);
	}

	function jsSnippet(alt: number): string {
		const meters = Math.round(alt * 1000);
		const formatted = meters.toString().replace(/\B(?=(\d{3})+(?!\d))/g, '_');
		return `const orbit = Keplerian.circular(${formatted});\n` + `orbit.orbitalPeriod();`;
	}

	function onAltitudeChange() {
		if (rustCode) rustCode.update`${rustSnippet(altitude)}`;
		if (pythonCode) pythonCode.update`${pythonSnippet(altitude)}`;
		if (jsCode) jsCode.update`${jsSnippet(altitude)}`;
	}
</script>

<Slide class="h-full place-content-center place-items-start px-16">
	<div class="flex w-full flex-col divide-y divide-zinc-700">
		<div class="grid grid-cols-[8rem_1fr] items-start gap-4 py-4 text-left">
			<p class="pt-1 text-left text-2xl font-bold text-zinc-300">Rust</p>
			<Code
				bind:this={rustCode}
				lang="rust"
				theme="poimandres"
				code={rustSnippet(altitude)}
				autoIndent={false}
				options={{ duration: 0, stagger: 0, containerStyle: false }}
			/>
		</div>
		<div class="grid grid-cols-[8rem_1fr] items-start gap-4 py-4 text-left">
			<p class="pt-1 text-left text-2xl font-bold text-zinc-300">Python</p>
			<Code
				bind:this={pythonCode}
				lang="python"
				theme="poimandres"
				code={pythonSnippet(altitude)}
				autoIndent={false}
				options={{ duration: 0, stagger: 0, containerStyle: false }}
			/>
		</div>
		<div class="grid grid-cols-[8rem_1fr] items-start gap-4 py-4 text-left">
			<p class="pt-1 text-left text-2xl font-bold text-zinc-300">JavaScript</p>
			<Code
				bind:this={jsCode}
				lang="javascript"
				theme="poimandres"
				code={jsSnippet(altitude)}
				autoIndent={false}
				options={{ duration: 0, stagger: 0, containerStyle: false }}
			/>
		</div>
	</div>

	<Transition>
		<div class="mt-8 flex flex-col items-center gap-4">
			<input
				type="range"
				min="200"
				max="36000"
				step="10"
				bind:value={altitude}
				oninput={onAltitudeChange}
				class="w-2/3 accent-sky-400"
			/>
			<p class="text-lg text-zinc-400">
				Altitude: <span class="font-mono text-white">{Math.round(altitude)} km</span>
			</p>
		</div>
	</Transition>

	<p class="mt-6 text-center text-2xl">
		Orbital period: <span class="font-mono font-bold text-sky-400"
			>{(period / 60).toFixed(1)} min</span
		>
	</p>

	<Notes>
		This slide shows the same operation — creating a circular orbit and getting its period — in all
		three languages. The code is live: the slider changes the altitude and recomputes the period via
		WebAssembly in real time.
	</Notes>
</Slide>
