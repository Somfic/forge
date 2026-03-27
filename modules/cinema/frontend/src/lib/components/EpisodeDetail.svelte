<script lang="ts">
	import type { Season, Episode, WatchHistoryItem } from "$lib/api.gen";
	import { imageUrl } from "$lib/utils";
	import { Button, MediaCard, Text } from "glow";
	import PlayCard from "./PlayCard.svelte";
	import DownloadButton from "./DownloadButton.svelte";

	let {
		season,
		episode,
		showTitle,
		tmdbId,
		resumeEntry,
		loadingStreams = false,
		onback,
		onselectepisode,
		onplay,
	}: {
		season: Season;
		episode: Episode;
		showTitle: string;
		tmdbId: number;
		resumeEntry?: WatchHistoryItem | null;
		loadingStreams?: boolean;
		onback: () => void;
		onselectepisode: (season: number, episode: number) => void;
		onplay?: () => void;
	} = $props();

	const canResume = $derived(
		resumeEntry?.season === season.season_number &&
		resumeEntry?.episode === episode.episode_number &&
		resumeEntry?.info_hash &&
		resumeEntry?.progress > 0
	);

	const remainingMin = $derived(
		canResume && resumeEntry ? Math.ceil((resumeEntry.duration - resumeEntry.progress) / 60) : null
	);

	const progressPct = $derived(
		canResume && resumeEntry ? (resumeEntry.progress / resumeEntry.duration) * 100 : 0
	);
</script>

<div class="episode-strip">
	<Button variant="ghost" icon="ArrowLeft" onclick={onback} />
	{#each season.episodes as ep}
		<MediaCard
			src={ep.still_path ? imageUrl(ep.still_path, "w185") : ""}
			aspectRatio="16/9"
			onclick={() => onselectepisode(season.season_number, ep.episode_number)}
		>
			{#snippet bottomLeft()}
				<Text size="xs" variant="muted">S{season.season_number} E{ep.episode_number}</Text>
			{/snippet}
		</MediaCard>
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

	{#if onplay}
		<PlayCard
			image={episode.still_path ? imageUrl(episode.still_path, "w780") : undefined}
			label="S{season.season_number} E{episode.episode_number}"
			action={canResume ? "Continue" : episode.name ?? "Play"}
			remaining={remainingMin ? `${remainingMin} min left` : undefined}
			progress={progressPct}
			loading={loadingStreams}
			onclick={onplay}
		/>
	{/if}

	<DownloadButton
		mediaType="tv"
		{tmdbId}
		title={showTitle}
		season={season.season_number}
		episode={episode.episode_number}
	/>
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

	@media (max-width: 768px) {
		.episode-strip {
			width: 100%;
			height: auto;
			max-height: 30vh;
			flex-direction: row;
			padding: 0.5rem;
			padding-top: 0.5rem;
			overflow-x: auto;
			overflow-y: hidden;
		}

		.episode-strip :global(> *) {
			width: 100px;
			flex-shrink: 0;
		}

		.sidebar {
			width: 100%;
			height: auto;
			padding: 1rem;
			padding-top: 1rem;
		}

		.ep-title {
			font-size: 1.2rem;
		}
	}
</style>
