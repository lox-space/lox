<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide } from '@animotion/core';
	import rwir from '$lib/assets/rwir.png';
</script>

<Slide class="h-full place-content-center place-items-center">
	<img src={rwir} alt="Rewrite it in Rust" class="max-h-[80vh] max-w-full" />

	<Notes>
		I had waited too long for such an opportunity and could not let the project fail so I took the
		lead and built a new team. We were then faced with the dilemma of how to replace poliastro and
		boiled down to two options. The reasonable choice: ship a JVM and use the industry standard,
		Orekit. Or the unreasonable choice: rewrite it in Rust. In the end we did both and this was the
		beginning of Lox.
	</Notes>
</Slide>
