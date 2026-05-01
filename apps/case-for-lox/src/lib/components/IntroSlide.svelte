<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide } from '@animotion/core';
	import QRCode from 'qrcode';

	const REPO_URL = 'https://github.com/lox-space/lox';

	const qrDataUrl = await QRCode.toDataURL(REPO_URL, {
		width: 200,
		margin: 1,
		color: { dark: '#f4f4f5', light: '#00000000' }
	});
</script>

<Slide class="h-full place-content-center">
	<div class="grid grid-cols-[1fr_auto] items-center gap-16">
		<div class="text-left">
			<p class="text-5xl font-bold">What is Lox?</p>
			<ul class="mt-8 space-y-3 text-2xl text-zinc-300">
				<li>
					Astrodynamics &amp; mission analysis framework written in <span
						class="font-semibold text-white">Rust</span
					>
				</li>
				<li>
					Bindings for <span class="font-semibold text-white">Python</span> and
					<span class="font-semibold text-white">WebAssembly</span>
				</li>
				<li>Time scales &amp; reference frames</li>
				<li>Orbital propagation &amp; analysis</li>
				<li>RF link budgets</li>
				<li class="text-xl text-zinc-400">Open source · MPL 2.0</li>
			</ul>
		</div>
		<div class="flex flex-col items-center gap-3">
			<img src={qrDataUrl} alt="QR code for github.com/lox-space/lox" class="h-48 w-48" />
			<p class="font-mono text-sm text-zinc-400">github.com/lox-space/lox</p>
		</div>
	</div>

	<Notes>
		A colleague once quipped that every time an aerospace engineer gets bored they start writing a
		new astrodynamics library. My goal today is to convince you that Lox is more than yet another
		library. And since this is at least my fourth attempt at writing such a library, I am getting
		very tired of repeating myself. Yet, I am confident that this will be the last time for the
		foreseeable future – famous last words – because thanks to Rust Lox runs everywhere that Rust
		runs. In Hyper, we are already using Lox on the server, in Jupiter notebooks through the
		comprehensive Python API, and we are starting to use it on the client as well by compiling to
		WebAssembly.
	</Notes>
</Slide>
