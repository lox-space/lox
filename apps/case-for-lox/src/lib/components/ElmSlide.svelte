<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide, Transition } from '@animotion/core';
</script>

<style>
	:global(.reveal table.no-borders th),
	:global(.reveal table.no-borders td) {
		border-bottom: none;
	}
</style>

<Slide class="h-full place-content-center place-items-center">
	<div class="flex items-center gap-20">
		<!-- Elm logo -->
		<img src="/Elm_logo.svg" alt="Elm logo" width="200" height="200" />

		<!-- Table fades in -->
		<Transition>
			<table class="no-borders text-2xl border-spacing-x-8 border-spacing-y-3" style="border-collapse: separate;">
				<thead>
					<tr class="text-left">
						<th class="font-normal text-zinc-400"></th>
						<th class="font-bold" style="color: #60b5cc;">Elm</th>
						<th class="font-bold" style="color: #f74c00;">Rust</th>
					</tr>
				</thead>
				<tbody class="text-zinc-300">
					<tr>
						<td>Algebraic data types</td>
						<td class="text-center text-green-400">✓</td>
						<td class="text-center text-green-400">✓</td>
					</tr>
					<tr>
						<td>No-configuration formatter</td>
						<td class="text-center text-green-400">✓</td>
						<td class="text-center text-green-400">✓</td>
					</tr>
					<tr>
						<td>Friendly compiler errors</td>
						<td class="text-center text-green-400">✓</td>
						<td class="text-center text-green-400">✓</td>
					</tr>
					<tr>
						<td>No runtime exceptions</td>
						<td class="text-center text-green-400">✓</td>
						<td class="text-center text-green-400">✓</td>
					</tr>
					<tr>
						<td class="font-bold text-white">Performance</td>
						<td class="text-center text-red-500">✗</td>
						<td class="text-center text-green-400">✓</td>
					</tr>
				</tbody>
			</table>
		</Transition>
	</div>

	<Notes>
		After LSIM, I went back to the drawing board and started looking into alternatives for Julia.
		While learning how to write web applications in preparation for building LSIM's GUI, I had found
		a somewhat esoteric frontend language called Elm. Show of hands who has heard of Elm or even
		used it in anger? If you haven't, you should give it a try. It is a mind-expanding experience. A
		glimpse of a parallel universe where frontend development makes sense. It had its moment in the
		spotlight a few years ago and then faded from view again due to the idiosyncrasies and idealism
		of its creator. Nevertheless it made a lasting impression on me and many other people in the
		industry — it was a major influence on the design of modern React for example. It also had great
		features and tooling such as algebraic data types or a no-configuration code formatter which I
		started to sorely miss in other languages and their ecosystems. Until I realized that Rust ticked
		these boxes as well and offered great performance to boot.
	</Notes>
</Slide>
