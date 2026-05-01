<script lang="ts">
	import '../styles/app.css';

	let { children } = $props();
</script>

<svelte:head>
	<title>The Case for Lox</title>
</svelte:head>

{@render children()}
