<script lang="ts">
	import type { MediaItem, Stream } from "$lib/api.gen";
	import { imageUrl } from "$lib/utils";
	import { Button, MediaCard, Pill, Text } from "glow";
	import StreamList from "./StreamList.svelte";

	let {
		item,
		streams = [],
		loadingStreams = false,
		onwatch,
		onplay,
		onselectseason,
	}: {
		item: MediaItem;
		streams?: Stream[];
		loadingStreams?: boolean;
		onwatch?: () => void;
		onplay?: (stream: Stream) => void;
		onselectseason?: (seasonNumber: number) => void;
	} = $props();
</script>

<div class="sidebar">
	{#if item.logo_path}
		<img
			class="logo"
			src={imageUrl(item.logo_path, "original")}
			alt={item.title}
		/>
	{:else}
		<h1 class="title">{item.title}</h1>
	{/if}

	<div class="meta">
		{#if item.release_date}
			<Text as="span" variant="secondary" size="sm">{item.release_date.slice(0, 4)}</Text>
		{/if}
		{#if item.runtime}
			<Text as="span" variant="secondary" size="sm">{item.runtime} min</Text>
		{/if}
		{#if item.seasons}
			<Text as="span" variant="secondary" size="sm">
				{item.seasons.length} season{item.seasons.length > 1 ? "s" : ""}
			</Text>
		{/if}
		{#if item.rating}
			<Text as="span" variant="secondary" size="sm">★ {item.rating.toFixed(1)}</Text>
		{/if}
	</div>

	{#if item.tagline}
		<Text variant="muted" size="sm">{item.tagline}</Text>
	{/if}

	{#if item.genres.length}
		<div class="genres">
			{#each item.genres as genre}
				<Pill label={genre.name} />
			{/each}
		</div>
	{/if}

	{#if item.overview}
		<Text size="sm">{item.overview}</Text>
	{/if}

	{#if item.media_type === "movie" && streams.length === 0 && onwatch}
		<Button
			variant="primary"
			label={loadingStreams ? "Finding streams..." : "Watch"}
			disabled={loadingStreams}
			loading={loadingStreams}
			onclick={onwatch}
		/>
	{/if}

	{#if item.media_type === "movie" && streams.length > 0 && onplay}
		<div class="streams-section">
			<Text weight="semibold" size="sm">Streams</Text>
			<StreamList {streams} {onplay} />
		</div>
	{/if}

	{#if item.seasons?.length && onselectseason}
		<div class="season-cards">
			{#each item.seasons as season}
				<MediaCard
					title={season.name}
					src={season.poster_path ? imageUrl(season.poster_path, "w185") : ""}
					aspectRatio="2/3"
					onclick={() => onselectseason(season.season_number)}
				/>
			{/each}
		</div>
	{/if}
</div>

<style>
	.sidebar {
		width: 40vw;
		min-height: 100vh;
		padding: 2rem;
		padding-top: 15vh;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.logo {
		object-fit: contain;
		max-width: 100%;
		filter: drop-shadow(0 2px 12px rgba(0, 0, 0, 0.6));
	}

	.title {
		font-size: 2.5rem;
		font-weight: 700;
		color: white;
		text-shadow: 0 2px 12px rgba(0, 0, 0, 0.5);
		margin: 0;
	}

	.meta {
		display: flex;
		gap: 0.75rem;
		align-items: center;
	}

	.genres {
		display: flex;
		gap: 0.4rem;
		flex-wrap: wrap;
	}

	.streams-section {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.season-cards {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
		gap: 0.5rem;
		margin-top: auto;
	}

	@media (max-width: 768px) {
		.sidebar {
			width: 100%;
			padding-top: 30vh;
		}
	}
</style>
