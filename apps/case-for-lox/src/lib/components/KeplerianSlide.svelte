<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

<script lang="ts">
	import { Slide, Notes } from '@animotion/core';
	import { T, Canvas } from '@threlte/core';
	import { Gizmo, OrbitControls } from '@threlte/extras';
	import { WebGLRenderer } from 'three';
	import { Earth, KeplerianOrbit } from '@lox-space/threlte';
	import { KeplerianSettings } from '@lox-space/svelte';

	let semiMajorAxis = $state(24464);
	let eccentricity = $state(0.7311);
	let inclination = $state(7.0);
	let raan = $state(57.7);
	let argPeriapsis = $state(178.1);
	let trueAnomaly = $state(25.4);
	let withEquatorialPlane = $state(false);
	let withOrbitalPlane = $state(false);
	let color = $state('#e92093');
</script>

<Slide class="h-full p-0">
	<div class="absolute inset-0">
		<Canvas
			createRenderer={(canvas) => {
				return new WebGLRenderer({
					canvas,
					logarithmicDepthBuffer: true
				});
			}}
		>
			<T.GridHelper args={[1e5, 1e1]} visible={withEquatorialPlane} />

			<T.PerspectiveCamera makeDefault position={[0, 0, 7e4]} far={1e12}>
				<OrbitControls>
					<Gizmo
						xColor="#ff4060"
						yColor="#40ff60"
						zColor="#4060ff"
						labelX="X"
						labelY="Z"
						labelZ="-Y"
					/>
				</OrbitControls>
			</T.PerspectiveCamera>

			<T.AmbientLight intensity={2} />

			<Earth textureUrl="/Earth-color.jpg" />
			<KeplerianOrbit
				{semiMajorAxis}
				{eccentricity}
				{inclination}
				{raan}
				{argPeriapsis}
				{trueAnomaly}
				{color}
				{withOrbitalPlane}
				name="Sat1"
			/>
		</Canvas>
	</div>

	<KeplerianSettings
		bind:semiMajorAxis
		bind:eccentricity
		bind:inclination
		bind:raan
		bind:argPeriapsis
		bind:trueAnomaly
		bind:color
		bind:withEquatorialPlane
		bind:withOrbitalPlane
	/>

	<Notes>
		Interactive 3D Keplerian orbit visualization. Drag to rotate, scroll to zoom. Adjust orbital
		elements with the panel on the right.
	</Notes>
</Slide>
