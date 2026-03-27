<script lang="ts">
	import type {
		MediaItem,
		Stream,
		SearchResult,
		WatchHistoryItem,
	} from "$lib/api.gen";
	import {
		addToCollection,
		removeFromCollection,
		isInCollection,
	} from "$lib/api.gen";
	import { imageUrl } from "$lib/utils";
	import { Button, Icon, MediaCard, Pill, Text } from "glow";
	import PlayCard from "./PlayCard.svelte";

	let {
		item,
		loadingStreams = false,
		onwatch,
		onselectseason,
		onselectepisode,
		similarItems = [],
		resumeEntry,
		onresume,
	}: {
		item: MediaItem;
		loadingStreams?: boolean;
		onwatch?: () => void;
		onselectseason?: (seasonNumber: number) => void;
		onselectepisode?: (season: number, episode: number) => void;
		similarItems?: SearchResult[];
		resumeEntry?: WatchHistoryItem | null;
		onresume?: () => void;
	} = $props();

	let onWatchlist = $state(false);
	let isFavorite = $state(false);
	let isWatched = $state(false);
	let watchlistLoading = $state(false);
	let favoriteLoading = $state(false);
	let watchedLoading = $state(false);

	$effect(() => {
		isInCollection("watchlist", item.media_type, item.id)
			.then((res) => {
				onWatchlist = res.data.in_collection;
			})
			.catch(() => {});
		isInCollection("favorites", item.media_type, item.id)
			.then((res) => {
				isFavorite = res.data.in_collection;
			})
			.catch(() => {});
		isInCollection("watched", item.media_type, item.id)
			.then((res) => {
				isWatched = res.data.in_collection;
			})
			.catch(() => {});
	});

	async function toggleCollection(
		name: string,
		current: boolean,
		setLoading: (v: boolean) => void,
		setState: (v: boolean) => void,
	) {
		setLoading(true);
		try {
			if (current) {
				await removeFromCollection(name, item.media_type, item.id);
				setState(false);
			} else {
				await addToCollection({
					collection: name,
					media_type: item.media_type,
					tmdb_id: item.id,
					title: item.title,
					poster_path: item.poster_path ?? undefined,
				});
				setState(true);
			}
		} catch {
		} finally {
			setLoading(false);
		}
	}
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
			<Text as="span" variant="secondary" size="sm"
				>{item.release_date.slice(0, 4)}
			</Text>
		{/if}
		{#if item.runtime}
			<Text as="span" variant="secondary" size="sm"
				>{item.runtime} min
			</Text>
		{/if}
		{#if item.seasons}
			<Text as="span" variant="secondary" size="sm">
				{item.seasons.length} season{item.seasons.length > 1 ? "s" : ""}
			</Text>
		{/if}
		{#if item.rating}
			<Text as="span" variant="secondary" size="sm">
				★ {item.rating.toFixed(1)}
			</Text>
		{/if}
	</div>

	<div class="actions-row">
		<span class:favorited={isFavorite}>
			<Button
				variant="ghost"
				icon="Heart"
				loading={favoriteLoading}
				onclick={() =>
					toggleCollection(
						"favorites",
						isFavorite,
						(v) => (favoriteLoading = v),
						(v) => (isFavorite = v),
					)}
			/>
		</span>
		<Button
			variant="ghost"
			icon={onWatchlist ? "BookmarkCheck" : "Bookmark"}
			loading={watchlistLoading}
			onclick={() =>
				toggleCollection(
					"watchlist",
					onWatchlist,
					(v) => (watchlistLoading = v),
					(v) => (onWatchlist = v),
				)}
		/>
		<Button
			variant="ghost"
			icon={isWatched ? "CircleCheck" : "Circle"}
			loading={watchedLoading}
			onclick={() =>
				toggleCollection(
					"watched",
					isWatched,
					(v) => (watchedLoading = v),
					(v) => (isWatched = v),
				)}
		/>
	</div>

	<!-- {#if item.overview}
		<Text size="sm">{item.overview}</Text>
	{/if} -->

	{#if item.media_type === "movie"}
		<div class="play-row">
			{#if resumeEntry?.info_hash && onresume}
				{@const remainingMin = Math.ceil(
					(resumeEntry.duration - resumeEntry.progress) / 60,
				)}
				<PlayCard
					image={item.backdrops?.[1]
						? imageUrl(item.backdrops[1], "w780")
						: item.backdrops?.[0]
							? imageUrl(item.backdrops[0], "w780")
							: undefined}
					label={item.title}
					action={item.tagline ?? "Continue"}
					remaining="{remainingMin} min left"
					progress={(resumeEntry.progress / resumeEntry.duration) *
						100}
					onclick={onresume}
				/>
			{:else if onwatch}
				<PlayCard
					image={item.backdrops?.[1]
						? imageUrl(item.backdrops[1], "w780")
						: item.backdrops?.[0]
							? imageUrl(item.backdrops[0], "w780")
							: undefined}
					label={item.title}
					action={item.tagline ?? "Watch"}
					loading={loadingStreams}
					onclick={onwatch}
				/>
			{/if}
		</div>
	{/if}

	{#if item.seasons?.length}
		<div class="play-row">
			{#if resumeEntry?.info_hash && onresume}
				{@const remainingMin = Math.ceil(
					(resumeEntry.duration - resumeEntry.progress) / 60,
				)}
				{@const resumeEpisode = item.seasons
					?.find((s) => s.season_number === resumeEntry.season)
					?.episodes?.find(
						(e) => e.episode_number === resumeEntry.episode,
					)}
				<PlayCard
					image={resumeEpisode?.still_path
						? imageUrl(resumeEpisode.still_path, "w780")
						: undefined}
					label={`S${resumeEntry.season} E${resumeEntry.episode}`}
					action={resumeEpisode?.name ?? "Continue"}
					remaining="{remainingMin} min left"
					progress={(resumeEntry.progress / resumeEntry.duration) *
						100}
					onclick={onresume}
				/>
			{:else if item.seasons[0]?.episodes?.[0] && onselectepisode}
				{@const firstSe = item.seasons[0]}
				{@const firstEp = firstSe.episodes[0]}
				<PlayCard
					image={firstEp.still_path
						? imageUrl(firstEp.still_path, "w780")
						: undefined}
					label={`S${firstSe.season_number} E${firstEp.episode_number}`}
					action={firstEp.name ?? "Continue"}
					onclick={() =>
						onselectepisode(
							item.seasons![0].season_number,
							firstEp.episode_number,
						)}
				/>
			{/if}

			{#if onselectseason}
				<Button
					variant="ghost"
					icon="LayoutGrid"
					onclick={() =>
						onselectseason(item.seasons![0].season_number)}
				/>
			{/if}
		</div>
	{/if}

	<!-- {#if similarItems.length > 0}
		<div class="similar-section">
			<Text weight="semibold" size="sm">Similar</Text>
			<div class="similar-grid">
				{#each similarItems.slice(0, 8) as sim}
					<MediaCard
						src={sim.poster_path
							? imageUrl(sim.poster_path, "w185")
							: ""}
						aspectRatio="2/3"
						onclick={() =>
							(window.location.href = `/cinema/${sim.media_type}/${sim.id}`)}
					>
						{#snippet bottomLeft()}
							<Text size="xs" variant="on-image">{sim.title}</Text>
						{/snippet}
					</MediaCard>
				{/each}
			</div>
		</div>
	{/if} -->
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
		justify-content: center;
	}

	/* ── Play row ── */
	.play-row {
		display: flex;
		gap: 0.5rem;
		margin-top: auto;
		align-self: flex-end;
		align-items: stretch;
		max-width: 100%;
	}

	.favorited :global(svg) {
		color: #ef4444;
		fill: #ef4444;
	}

	.actions-row {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.genres {
		display: flex;
		gap: 0.4rem;
		flex-wrap: wrap;
	}

	@media (max-width: 768px) {
		.sidebar {
			width: 100%;
			padding-top: 30vh;
		}
	}
</style>
