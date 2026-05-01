<script lang="ts">
	import { Presentation } from '@animotion/core';
	import OrbitSlide from '$lib/components/OrbitSlide.svelte';
	import KeplerianSlide from '$lib/components/KeplerianSlide.svelte';
	import TitleSlide from '$lib/components/TitleSlide.svelte';
	import IntroSlide from '$lib/components/IntroSlide.svelte';
	import LoxNameSlide from '$lib/components/LoxNameSlide.svelte';
	import EsocSlide from '$lib/components/EsocSlide.svelte';
	import RequirementsSlide from '$lib/components/RequirementsSlide.svelte';
	import TradeOffSlide from '$lib/components/TradeOffSlide.svelte';
	import StudySlide from '$lib/components/StudySlide.svelte';
	import TimeSlide from '$lib/components/TimeSlide.svelte';
	import GUISlide from '$lib/components/GUISlide.svelte';
	import LsimSlide from '$lib/components/LsimSlide.svelte';
	import ElmSlide from '$lib/components/ElmSlide.svelte';
	import EphemeristaSlide from '$lib/components/EphemeristaSlide.svelte';
	import PoliastroDeadSlide from '$lib/components/PoliastroDeadSlide.svelte';
	import RwirSlide from '$lib/components/RwirSlide.svelte';
	import ArchitectureSlide from '$lib/components/ArchitectureSlide.svelte';
	import FrameworkSlide from '$lib/components/FrameworkSlide.svelte';
	import ReplicaSlide from '$lib/components/ReplicaSlide.svelte';
	import LessonsSlide from '$lib/components/LessonsSlide.svelte';
	import ReplicaHorizonSlide from '$lib/components/ReplicaHorizonSlide.svelte';
	import ClosingSlide from '$lib/components/ClosingSlide.svelte';
</script>

<Presentation
	class="bg-lox"
	options={{ history: true, transition: 'slide', controls: false, progress: true }}
>
	<TitleSlide />
	<IntroSlide />
	<LoxNameSlide />
	<OrbitSlide />
	<EsocSlide />
	<RequirementsSlide />
	<TradeOffSlide />
	<StudySlide />
	<TimeSlide />
	<GUISlide />
	<LsimSlide />
	<KeplerianSlide />
	<ElmSlide />
	<EphemeristaSlide />
	<PoliastroDeadSlide />
	<RwirSlide />
	<ArchitectureSlide />
	<FrameworkSlide />
	<ReplicaSlide />
	<ReplicaHorizonSlide />
	<LessonsSlide />
	<ClosingSlide />
</Presentation>
