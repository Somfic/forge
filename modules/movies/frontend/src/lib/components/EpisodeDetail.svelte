<script lang="ts">
	import type { Season, Stream, Episode } from "$lib/api.gen";
	import { imageUrl } from "$lib/utils";
	import { Button, MediaCard, Text } from "glow";
	import StreamList from "./StreamList.svelte";

	let {
		season,
		episode,
		showTitle,
		streams = [],
		loadingStreams = false,
		onback,
		onselectepisode,
		onplay,
	}: {
		season: Season;
		episode: Episode;
		showTitle: string;
		streams?: Stream[];
		loadingStreams?: boolean;
		onback: () => void;
		onselectepisode: (season: number, episode: number) => void;
		onplay?: (stream: Stream) => void;
	} = $props();
</script>

<div class="episode-strip">
	<Button variant="ghost" icon="ArrowLeft" onclick={onback} />
	{#each season.episodes as ep}
		<MediaCard
			title="S{season.season_number} E{ep.episode_number}"
			src={ep.still_path ? imageUrl(ep.still_path, "w185") : ""}
			aspectRatio="16/9"
			onclick={() => onselectepisode(season.season_number, ep.episode_number)}
		/>
	{/each}
</div>

<div class="sidebar">
	<h2 class="ep-title">{episode.name}</h2>
	<div class="meta">
		<Text as="span" variant="secondary" size="sm">
			S{season.season_number} E{episode.episode_number}
		</Text>
		<Text as="span" variant="secondary" size="sm">{showTitle}</Text>
	</div>

	{#if episode.overview}
		<Text size="sm">{episode.overview}</Text>
	{/if}

	{#if loadingStreams}
		<Text variant="muted" size="sm">Finding streams...</Text>
	{/if}

	{#if streams.length > 0 && onplay}
		<div class="streams-section">
			<Text weight="semibold" size="sm">Streams</Text>
			<StreamList {streams} {onplay} />
		</div>
	{/if}
</div>

<style>
	.episode-strip {
		width: 140px;
		flex-shrink: 0;
		padding: 1rem 0.5rem;
		padding-top: 2rem;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		overflow-y: auto;
		background: rgba(0, 0, 0, 0.3);
		backdrop-filter: blur(12px);
	}

	.episode-strip :global(> *) {
		width: 120px;
		flex-shrink: 0;
	}

	.sidebar {
		width: 35vw;
		height: 100vh;
		overflow-y: auto;
		padding: 2rem;
		padding-top: 15vh;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.ep-title {
		font-size: 1.5rem;
		font-weight: 700;
		color: white;
		margin: 0;
	}

	.meta {
		display: flex;
		gap: 0.75rem;
		align-items: center;
	}

	.streams-section {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}
</style>
