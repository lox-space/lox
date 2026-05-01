<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide, Transition } from '@animotion/core';
	import ephemerista from '$lib/assets/ephemerista.png';
</script>

<Slide class="h-full place-content-center place-items-center bg-black">
	<img src={ephemerista} alt="Ephemerista logo" class="w-[700px]" />

	<Transition class="mt-4 text-xl text-zinc-400 italic">"Murphy's Law, the project"</Transition>

	<Notes>
		But I wasn't ready to "rewrite it in Rust" just yet. Through some twists of fate I joined an ESA
		project that was managed by the Greek Libre Space Foundation (LSF), a non-profit building open
		source space hardware and software, as a part-time freelancer. The project would eventually be
		called Ephemerista or as I like to call it "Murphy's Law, the project". The mission was to
		develop a new open source mission analysis simulator for telecommunications constellations based
		on existing open source technologies for the European Space Agency and do it in 18 months.
		Things did not go to plan. One of the main requirements was to provide a Python API and thus we
		chose „poliastro“, the leading open source astrodynamics library written in Python, as the
		centrepiece of the implementation. We also had the current and the former maintainer and creator
		of poliastro on the team. In August 2023 we were in month number six of our two month
		requirement analysis phase and then this happened.
	</Notes>
</Slide>
