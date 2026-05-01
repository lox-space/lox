<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide } from '@animotion/core';
	import { onMount } from 'svelte';
	import initWasm, { Utc, WasmTime } from '@lox-space/wasm';

	await initWasm();

	const SCALES = ['TAI', 'TT', 'TDB', 'TCG', 'TCB'] as const;

	type Row = { scale: string; time: WasmTime };

	let utc = $state('');
	let rows = $state<Row[]>([]);

	function tick() {
		const now = new Date();
		// toISOString gives "2026-04-28T10:30:45.123Z" — strip the Z
		const iso = now.toISOString().replace('Z', '');
		const u = Utc.fromIso(iso);
		utc = u.toString();
		rows = SCALES.map((scale) => ({ scale, time: u.toScale(scale) }));
	}

	onMount(() => {
		tick();
		const id = setInterval(tick, 1000);
		return () => clearInterval(id);
	});
</script>

<Slide class="h-full place-content-center place-items-center">
	<p class="text-4xl font-bold">Time Systems</p>

	<div class="mt-10 w-full max-w-2xl divide-y divide-zinc-800 font-mono">
		<div class="grid grid-cols-[6rem_1fr] gap-4 py-3 text-xl">
			<span class="font-bold text-sky-400">UTC</span>
			<span class="text-white">{utc}</span>
		</div>
		{#each rows as { scale, time }}
			<div class="grid grid-cols-[6rem_1fr] gap-4 py-3 text-xl">
				<span class="font-bold text-zinc-400">{scale}</span>
				<span class="text-zinc-200">{time.toString()}</span>
			</div>
		{/each}
	</div>

	<Notes>
		Turns out that reality is surprisingly complicated. Which is one of the reasons that most of
		these bored engineer projects do not go anywhere. Implementing a few algorithms from Vallado is
		not enough for a production-ready system.

		My canonical example is time systems. You start out with UTC and TAI which is already annoying
		due to leap seconds. But soon you are dealing with hypothetical clocks at the barycentre of the
		Solar system which is quite the rabbit hole.
	</Notes>
</Slide>
