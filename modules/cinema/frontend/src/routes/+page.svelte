<script lang="ts">
	import { search, type SearchResult } from "$lib/api.gen";
	import { imageUrl } from "$lib/utils";
	import { Heading, Input, MediaCard, Text } from "glow";

	let query = $state("");
	let results = $state<SearchResult[]>([]);
	let loading = $state(false);
	let timeout: ReturnType<typeof setTimeout>;

	function onInput() {
		clearTimeout(timeout);
		if (query.length < 2) {
			results = [];
			return;
		}
		timeout = setTimeout(async () => {
			loading = true;
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
	<Heading level={1}>Movies & TV</Heading>

	<Input
		type="text"
		placeholder="Search movies and TV shows..."
		value={query}
		icon={loading ? "LoaderCircle" : "Search"}
		onChange={(v) => {
			query = v;
			onInput();
		}}
	/>

	{#if results.length > 0}
		<div class="grid">
			{#each results as item}
				<MediaCard
					title={item.title}
					subtitle={item.release_date
						? `${item.release_date.slice(0, 4)} • ${
								item.media_type === "movie" ? "Movie" : "TV"
							}`
						: item.media_type === "movie"
							? "Movie"
							: "TV"}
					src={item.poster_path
						? imageUrl(item.poster_path, "w342")
						: ""}
					aspectRatio="2/3"
					onclick={() =>
						(window.location.href = `/cinema/${item.media_type}/${item.id}`)}
				/>
			{/each}
		</div>
	{/if}
</div>

<style>
	.content {
		padding: 2rem;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		gap: 1rem;
		margin-top: 1rem;
	}
</style>
