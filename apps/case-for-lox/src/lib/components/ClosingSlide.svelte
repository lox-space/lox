<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide } from '@animotion/core';
	import loxLogo from '$lib/assets/lox-patch.png';
	import QRCode from 'qrcode';

	const REPO_URL = 'https://github.com/lox-space/lox';

	const qrDataUrl = await QRCode.toDataURL(REPO_URL, {
		width: 200,
		margin: 1,
		color: { dark: '#f4f4f5', light: '#00000000' }
	});
</script>

<Slide class="h-full place-content-center place-items-center">
	<div class="flex w-full max-w-4xl items-center gap-16">
		<img src={loxLogo} alt="Lox logo" class="h-72 shrink-0 rounded-2xl bg-white" />

		<div class="flex flex-col items-center gap-6">
			<p class="text-5xl font-bold">Thank You!</p>
			<div class="flex flex-col items-center gap-3">
				<img src={qrDataUrl} alt="QR code for github.com/lox-space/lox" class="h-48 w-48" />
				<p class="font-mono text-zinc-400">github.com/lox-space/lox</p>
			</div>
		</div>
	</div>
</Slide>
