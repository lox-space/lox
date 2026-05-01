<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide, Transition } from '@animotion/core';
</script>

<Slide class="h-full place-content-center place-items-center">
	<p class="text-lg tracking-widest text-zinc-500 uppercase">ESOC · Mission Analysis · 2012</p>

	<Transition class="mt-12">
		<p class="font-mono text-8xl font-bold tracking-tight text-white">pvecle</p>
	</Transition>

	<Transition class="mt-8">
		<pre class="rounded-lg bg-zinc-900 px-8 py-6 text-left font-mono text-lg leading-relaxed text-zinc-300">      SUBROUTINE PVECLE(MU, R, V, A, E, XI, OM, W, TA)
      IMPLICIT REAL*8 (A-H, O-Z)
      REAL*8 R(3), V(3)
C     ANGULAR MOMENTUM VECTOR
      HX = R(2)*V(3) - R(3)*V(2)
      HY = R(3)*V(1) - R(1)*V(3)
      HZ = R(1)*V(2) - R(2)*V(1)
      H  = DSQRT(HX**2 + HY**2 + HZ**2)
C     SEMI-MAJOR AXIS
      RR = DSQRT(R(1)**2 + R(2)**2 + R(3)**2)
      VV = DSQRT(V(1)**2 + V(2)**2 + V(3)**2)
      A  = 1.0D0 / (2.0D0/RR - VV**2/MU)</pre>
	</Transition>

	<Transition class="mt-6">
		<p class="font-mono text-8xl font-bold tracking-tight text-zinc-500">pelvec</p>
	</Transition>

	<Notes
		>As I mentioned, this ain't my first rodeo. So let's go back to the beginning. It's late 2012
		and I am a blue-eyed student intern starting my internship in the mission analysis section at
		ESOC wondering what kind of magnificent software they are using to design ESA's missions.

		To my astonishment, I learn that it is just a huge pile of Fortran. The kind of where the
		authors seem to not have realised that punch cards aren't a thing anymore and variable names can
		have more than six characters. Quite the shock, if you are used to Python or MATLAB.

		Anyone wants to guess what the pvecle subroutine is used for? If you guessed conversion from
		Cartesian state vectors to Keplerian elements for any body with known gravitational parameter,
		you guessed correct. The inverse operation is called pelvec of course.

		Another shocking insight is that Fortran is actually quite nice to work with if you use anything
		more modern than Fortran 77. Still I am left wondering if there is a better way. And since I get
		bored easily I start my own library.</Notes
	>
</Slide>
