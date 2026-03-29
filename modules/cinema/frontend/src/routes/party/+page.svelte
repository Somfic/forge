<script lang="ts">
	import { goto } from "$app/navigation";
	import { Button, Heading, Input, Text } from "glow";
	import { createRoom, joinRoom, party } from "$lib/watch-party.svelte";

	let joinCode = $state("");

	function create() {
		createRoom();
	}

	function join() {
		const code = joinCode.trim().toUpperCase();
		if (code.length >= 4) {
			joinRoom(code);
		}
	}

	// Once connected, go to the main cinema page to start browsing
	$effect(() => {
		if (party.active && (party.phase === 'lobby' || party.phase === 'picking')) {
			goto("/cinema");
		}
	});
</script>

<svelte:head>
	<title>Watch Party</title>
</svelte:head>

<div class="content">
	<div class="hero">
		<Heading level={1}>Watch Party</Heading>
		<Text variant="muted">Watch together with friends in perfect sync.</Text>
	</div>

	<div class="cards">
		<div class="card">
			<Heading level={3}>Create a room</Heading>
			<Text variant="muted" size="sm">Start a watch party and invite friends with a code.</Text>
			<Button label="Create Room" onclick={create} />
		</div>

		<div class="card">
			<Heading level={3}>Join a room</Heading>
			<Text variant="muted" size="sm">Enter a room code to join an existing party.</Text>
			<div class="join-row">
				<Input
					type="text"
					placeholder="Room code"
					value={joinCode}
					onChange={(v) => (joinCode = v)}
				/>
				<Button label="Join" onclick={join} disabled={joinCode.trim().length < 4} />
			</div>
		</div>
	</div>
</div>

<style>
	.content {
		padding: 2rem;
		display: flex;
		flex-direction: column;
		gap: 2rem;
		max-width: 640px;
		margin: 0 auto;
	}

	.hero {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		text-align: center;
		padding-top: 2rem;
	}

	.cards {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.card {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		padding: 1.5rem;
		border-radius: 12px;
		background: var(--color-surface, rgba(255, 255, 255, 0.05));
		border: 1px solid var(--color-border, rgba(255, 255, 255, 0.08));
	}

	.join-row {
		display: flex;
		gap: 0.5rem;
		align-items: flex-start;
	}
</style>
