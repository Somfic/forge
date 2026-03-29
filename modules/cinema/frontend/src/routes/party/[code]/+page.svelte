<script lang="ts">
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import { joinRoom, party } from "$lib/watch-party.svelte";
	import { Text } from "glow";

	const code = $derived((page.params.code ?? "").toUpperCase());

	// Join the room on mount
	$effect(() => {
		if (!party.active) {
			joinRoom(code);
		}
	});

	// Once connected, go to the main cinema page
	$effect(() => {
		if (party.active && party.phase !== 'connecting' && party.phase !== 'disconnected') {
			goto("/cinema");
		}
	});
</script>

<svelte:head>
	<title>Joining Watch Party...</title>
</svelte:head>

<div class="content">
	{#if party.phase === 'connecting'}
		<Text variant="muted">Joining room {code}...</Text>
	{:else if party.phase === 'disconnected'}
		<Text variant="muted">{party.error ?? "Failed to connect"}</Text>
	{/if}
</div>

<style>
	.content {
		display: flex;
		justify-content: center;
		align-items: center;
		padding: 4rem;
	}
</style>
