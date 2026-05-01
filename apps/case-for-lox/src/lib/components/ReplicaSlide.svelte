<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Notes, Slide } from '@animotion/core';
	import replicaHla from '$lib/assets/replica-hla.png';
</script>

<Slide class="h-full place-content-center place-items-center">
	<img src={replicaHla} alt="Replica high-level architecture" class="max-h-[85vh] max-w-full" />

	<Notes>
		So where did all this lead? Ephemerista shipped. And while the project had its share of Murphy's
		Law moments, it proved that Lox could carry a real product. On the back of that, I started a new
		project called Replica for KSAT — Kongsberg Satellite Services, one of the world's largest
		ground station operators. Replica is a mission analysis and flight dynamics platform for their
		Hyperion and Hyper constellations. You can see the architecture here: Replica crates extend Lox
		to model the specific constellations, a Rust web service exposes a REST API, a Python package
		enables Jupyter-based ad-hoc analyses, and frontend apps are built on Kerolox. Lox and Kerolox
		sit at the open source foundation — the proprietary Replica layer is just a relatively thin shell
		on top.
	</Notes>
</Slide>
