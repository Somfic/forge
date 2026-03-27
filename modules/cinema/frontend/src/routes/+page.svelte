<script lang="ts">
	import {
		search,
		trending as fetchTrending,
		watchHistory as fetchWatchHistory,
		getCollection as fetchCollection,
		type SearchResult,
		type WatchHistoryItem,
		type CollectionItem,
	} from "$lib/api.gen";
	import { imageUrl } from "$lib/utils";
	import { Heading, Input, MediaCard, Text } from "glow";

	let query = $state("");
	let results = $state<SearchResult[]>([]);
	let loading = $state(false);
	let timeout: ReturnType<typeof setTimeout>;

	let trendingItems = $state<SearchResult[]>([]);
	let historyItems = $state<WatchHistoryItem[]>([]);
	let watchlistItems = $state<CollectionItem[]>([]);
	let favoriteItems = $state<CollectionItem[]>([]);
	let watchedItems = $state<CollectionItem[]>([]);

	// Load browse data on mount
	$effect(() => {
		fetchWatchHistory()
			.then((res) => {
				historyItems = res.data;
			})
			.catch(() => {});
		fetchCollection("watchlist")
			.then((res) => {
				watchlistItems = res.data;
			})
			.catch(() => {});
		fetchCollection("favorites")
			.then((res) => {
				favoriteItems = res.data;
			})
			.catch(() => {});
		fetchCollection("watched")
			.then((res) => {
				watchedItems = res.data;
			})
			.catch(() => {});
		fetchTrending()
			.then((res) => {
				trendingItems = res.data;
			})
			.catch(() => {});
	});

	const browsing = $derived(query.length < 2 && results.length === 0);

	function onInput() {
		clearTimeout(timeout);
		if (query.length < 2) {
			results = [];
			return;
		}
		loading = true;
		timeout = setTimeout(async () => {
			try {
				const res = await search({ q: query });
				results = res.data;
			} finally {
				loading = false;
			}
		}, 300);
	}
</script>

<div class="content">
	<Input
		type="text"
		placeholder="Search movies and TV shows..."
		value={query}
		icon={"Search"}
		{loading}
		onChange={(v) => {
			query = v;
			onInput();
		}}
	/>

	{#if browsing}
		{#if historyItems.length > 0}
			<section>
				<Heading level={2}>Continue Watching</Heading>
				<div class="row">
					{#each historyItems as item}
						{@const minLeft = Math.ceil(
							(item.duration - item.progress) / 60,
						)}
						{@const pct =
							item.duration > 0
								? (item.progress / item.duration) * 100
								: 0}
						<MediaCard
							src={item.poster_path
								? imageUrl(item.poster_path, "w342")
								: ""}
							aspectRatio="2/3"
							progress={pct}
							onclick={() =>
								(window.location.href = `/cinema/${item.media_type}/${item.tmdb_id}`)}
						>
							{#snippet bottomLeft()}
								<Text size="xs" variant="muted">
									{item.media_type === "tv" && item.season > 0
										? `S${item.season} E${item.episode} · ${minLeft} min left`
										: `${minLeft} min left`}
								</Text>
							{/snippet}
						</MediaCard>
					{/each}
				</div>
			</section>
		{/if}

		{#if watchlistItems.length > 0}
			<section>
				<Heading level={2}>Watchlist</Heading>
				<div class="row">
					{#each watchlistItems as item}
						<MediaCard
							src={item.poster_path
								? imageUrl(item.poster_path, "w342")
								: ""}
							aspectRatio="2/3"
							onclick={() =>
								(window.location.href = `/cinema/${item.media_type}/${item.tmdb_id}`)}
						>
							{#snippet bottomLeft()}
								<Text size="xs" variant="muted">{item.title}</Text>
							{/snippet}
						</MediaCard>
					{/each}
				</div>
			</section>
		{/if}

		{#if favoriteItems.length > 0}
			<section>
				<Heading level={2}>Favorites</Heading>
				<div class="row">
					{#each favoriteItems as item}
						<MediaCard
							src={item.poster_path
								? imageUrl(item.poster_path, "w342")
								: ""}
							aspectRatio="2/3"
							onclick={() =>
								(window.location.href = `/cinema/${item.media_type}/${item.tmdb_id}`)}
						>
							{#snippet bottomLeft()}
								<Text size="xs" variant="muted">{item.title}</Text>
							{/snippet}
						</MediaCard>
					{/each}
				</div>
			</section>
		{/if}

		{#if trendingItems.length > 0}
			<section>
				<Heading level={2}>Trending This Week</Heading>
				<div class="row">
					{#each trendingItems as item}
						<MediaCard
							src={item.poster_path
								? imageUrl(item.poster_path, "w342")
								: ""}
							aspectRatio="2/3"
							onclick={() =>
								(window.location.href = `/cinema/${item.media_type}/${item.id}`)}
						>
							{#snippet bottomLeft()}
								<Text size="xs" variant="muted">{item.title}</Text>
							{/snippet}
							{#snippet bottomRight()}
								<Text size="xs" variant="muted">
									{item.release_date
										? `${item.release_date.slice(0, 4)} · ${item.media_type === "movie" ? "Movie" : "TV"}`
										: item.media_type === "movie"
											? "Movie"
											: "TV"}
								</Text>
							{/snippet}
						</MediaCard>
					{/each}
				</div>
			</section>
		{/if}

		{#if watchedItems.length > 0}
			<section>
				<Heading level={2}>Watched</Heading>
				<div class="row">
					{#each watchedItems as item}
						<MediaCard
							src={item.poster_path
								? imageUrl(item.poster_path, "w342")
								: ""}
							aspectRatio="2/3"
							onclick={() =>
								(window.location.href = `/cinema/${item.media_type}/${item.tmdb_id}`)}
						>
							{#snippet bottomLeft()}
								<Text size="xs" variant="muted">{item.title}</Text>
							{/snippet}
						</MediaCard>
					{/each}
				</div>
			</section>
		{/if}
	{:else if results.length > 0}
		<div class="grid">
			{#each results as item}
				<MediaCard
					src={item.poster_path
						? imageUrl(item.poster_path, "w342")
						: ""}
					aspectRatio="2/3"
					onclick={() =>
						(window.location.href = `/cinema/${item.media_type}/${item.id}`)}
				>
					{#snippet bottomLeft()}
						<Text size="xs" variant="muted">{item.title}</Text>
					{/snippet}
					{#snippet bottomRight()}
						<Text size="xs" variant="muted">
							{item.release_date
								? `${item.release_date.slice(0, 4)} · ${item.media_type === "movie" ? "Movie" : "TV"}`
								: item.media_type === "movie"
									? "Movie"
									: "TV"}
						</Text>
					{/snippet}
				</MediaCard>
			{/each}
		</div>
	{/if}
</div>

<style>
	.content {
		padding: 2rem;
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		gap: 1rem;
	}

	section {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.row {
		display: flex;
		gap: 0.75rem;
		overflow-x: auto;
		padding-bottom: 0.25rem;
	}

	.row :global(> *) {
		width: 160px;
		flex-shrink: 0;
	}
</style>
