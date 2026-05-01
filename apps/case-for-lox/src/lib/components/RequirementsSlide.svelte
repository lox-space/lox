<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide, Transition } from '@animotion/core';
</script>

<Slide class="h-full place-content-center place-items-center">
	<p class="text-5xl font-bold">Requirements</p>

	<ul class="mt-12 space-y-6 text-left text-3xl text-zinc-300">
		<Transition as="li">
			<span class="font-bold text-white">Performance</span>
			<span class="ml-3 text-zinc-400">— millions of simulations</span>
		</Transition>
		<Transition as="li">
			<span class="font-bold text-white">Flexibility</span>
			<span class="ml-3 text-zinc-400">— requirements change</span>
		</Transition>
		<Transition as="li">
			<span class="font-bold text-white">Interactivity</span>
			<span class="ml-3 text-zinc-400">— exploratory analyses</span>
		</Transition>
	</ul>

	<Notes>
		So what are my requirements for a great astrodynamics toolkit? First of all, performance. If we
		do numerical optimisation or Monte Carlo analyses, we may run our simulations millions of times.
		Thus, individual executions have to be fast or we'll have to wait forever. As you know, I am
		easily bored so we cannot have that. Secondly, our analysis tools need to be flexible because
		mission requirements and workflows change all the time. Finally, we want our tools to be
		interactive to enable exploratory analyses especially for early phase studies where we are not
		quite sure what we are building yet.
	</Notes>
</Slide>
