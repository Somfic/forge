<script lang="ts">
	import "glow/styles";
	import { afterNavigate, goto } from "$app/navigation";
	import { party, disconnect, navigate as partyNavigate } from "$lib/watch-party.svelte";
	import { Button, Text } from "glow";

	let { children } = $props();

	// Auto-sync all host navigations to guests
	afterNavigate(({ to }) => {
		if (party.active && party.role === 'host' && to?.url) {
			partyNavigate(to.url.pathname + to.url.search);
		}
	});

	// Navigate to player when watching phase starts
	$effect(() => {
		if (party.phase === 'watching' && party.content) {
			const c = party.content;
			let url = `/cinema/${c.media_type}/${c.tmdb_id}/play/${c.info_hash}/${c.file_idx}`;
			if (c.season != null && c.episode != null) {
				url += `?s=${c.season}&e=${c.episode}`;
			}
			goto(url);
		}
	});
</script>

{#if party.active && party.phase !== 'disconnected' && party.phase !== 'connecting'}
	<div class="party-bar">
		<div class="party-bar-left">
			<span class="dot"></span>
			<Text size="sm">
				Watch Party · <strong>{party.roomCode}</strong> · {party.participantCount} connected
			</Text>
		</div>

		{#if party.phase === 'lobby'}
			<Text size="sm" variant="muted">Waiting for friends...</Text>
		{:else if party.phase === 'picking'}
			{#if party.role === 'host'}
				<Text size="sm" variant="muted">Pick something to watch</Text>
			{:else}
				<Text size="sm" variant="muted">Host is choosing...</Text>
			{/if}
		{/if}

		<Button label="Leave" variant="ghost" onclick={disconnect} />
	</div>
{/if}

{@render children()}

<style>
	@font-face {
		font-family: 'Subtitle';
		src: url('/cinema/fonts/Helvetica Neue 67 Medium Condensed.otf') format('opentype');
		font-display: swap;
	}

	:global(:root) {
		--subtitle-font: 'Subtitle', system-ui, sans-serif;
		--party-bar-height: 0px;
	}

	:global(:root:has(.party-bar)) {
		--party-bar-height: 45px;
	}

	.party-bar {
		position: sticky;
		top: 0;
		z-index: 50;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
		padding: 0.5rem 1.5rem;
		background: rgba(0, 0, 0, 0.8);
		backdrop-filter: blur(12px);
		border-bottom: 1px solid rgba(255, 255, 255, 0.08);
	}

	.party-bar-left {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #4ade80;
		flex-shrink: 0;
		animation: pulse 2s infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.4; }
	}
</style>
