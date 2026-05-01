<script>
	import { Notes, Slide } from '@animotion/core';
	import logo from '$lib/assets/lox-logo.png';
</script>

<Slide class="h-full place-content-center">
	<div class="grid grid-cols-2 items-center gap-16">
		<div class="text-left">
			<p class="text-6xl font-bold">The Case for Lox</p>
			<p class="mt-4 text-3xl">Helge Eichhorn</p>
		</div>
		<img src={logo} alt="Lox logo" class="h-100 w-auto justify-self-center" />
	</div>
	<Notes
		>Hi, my name is Helge, I am a freelance astrodynamicist, and since late last year I have been
		consulting for KSAT to help build Hyper. I am very happy to be here because this means that the
		open source project which enables me to do this has suddenly become an overnight success and it
		took only a little more than a decade to get here.
	</Notes>
</Slide>
