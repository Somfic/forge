<script lang="ts">
	import { page } from "$app/state";
	import {
		getDetails,
		imageUrl,
		type MediaItem,
		type MediaType,
	} from "$lib/api";

	let item = $state<MediaItem | null>(null);
	let error = $state<string | null>(null);

	$effect(() => {
		const type = page.params.type as MediaType;
		const id = Number(page.params.id);
		item = null;
		error = null;
		getDetails(type, id)
			.then((res) => (item = res.data))
			.catch((e) => (error = e.message));
	});
</script>

{#if error}
	<p class="error">{error}</p>
{:else if !item}
	<p>Loading...</p>
{:else}
	{#if item.backdrop_path}
		<div
			class="backdrop"
			style="background-image: url({imageUrl(
				item.backdrop_path,
				'w1280',
			)})"
		>
			{#if item.images?.logos?.length}
				<img
					class="logo"
					src={imageUrl(item.images.logos[0].file_path, "w500")}
					alt={item.title}
				/>
			{:else}
				<h1>{item.title}</h1>
			{/if}
		</div>
	{:else}
		<h1>{item.title}</h1>
	{/if}

	{#if item.tagline}
		<p class="tagline">{item.tagline}</p>
	{/if}

	<div class="meta">
		<span class="badge" class:tv={item.media_type === "tv"}>
			{item.media_type === "movie" ? "Movie" : "TV"}
		</span>
		{#if item.release_date}
			<span>{item.release_date.slice(0, 4)}</span>
		{/if}
		{#if item.runtime}
			<span>{item.runtime} min</span>
		{/if}
		{#if item.rating}
			<span>★ {item.rating.toFixed(1)}</span>
		{/if}
	</div>

	{#if item.genres.length}
		<div class="genres">
			{#each item.genres as genre}
				<span class="chip">{genre.name}</span>
			{/each}
		</div>
	{/if}

	{#if item.overview}
		<p class="overview">{item.overview}</p>
	{/if}

	{#if item.seasons?.length}
		<h2>Seasons</h2>
		<div class="seasons">
			{#each item.seasons as season}
				<div class="season">
					{#if season.poster_path}
						<img
							src={imageUrl(season.poster_path, "w185")}
							alt={season.name}
						/>
					{/if}
					<h3>{season.name}</h3>
					<p>{season.episode_count} episodes</p>
				</div>
			{/each}
		</div>
	{/if}

	{#if item.videos?.length}
		<h2>Trailers</h2>
		<div class="trailers">
			{#each item.videos.filter((v) => v.site === "YouTube") as video}
				<div class="trailer">
					<iframe
						src="https://www.youtube.com/embed/{video.key}"
						title={video.name}
						allowfullscreen
					></iframe>
					<p>{video.name}</p>
				</div>
			{/each}
		</div>
	{/if}
{/if}

<style>
	.backdrop {
		width: 100%;
		aspect-ratio: 16/9;
		background-size: cover;
		background-position: center;
		border-radius: 12px;
		display: flex;
		align-items: flex-end;
		padding: 2rem;
		position: relative;
	}

	.backdrop::after {
		content: "";
		position: absolute;
		inset: 0;
		background: linear-gradient(transparent 40%, rgba(0, 0, 0, 0.8));
		border-radius: 12px;
	}

	.backdrop h1,
	.backdrop .logo {
		position: relative;
		z-index: 1;
		color: white;
	}

	.logo {
		max-width: 300px;
		max-height: 100px;
	}

	.tagline {
		font-style: italic;
		color: #888;
		margin: 0.5rem 0;
	}

	.meta {
		display: flex;
		gap: 1rem;
		align-items: center;
		color: #aaa;
		margin: 0.5rem 0;
	}

	.badge {
		background: #2563eb;
		color: white;
		padding: 0.15rem 0.5rem;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: bold;
	}

	.badge.tv {
		background: #7c3aed;
	}

	.genres {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
		margin: 0.5rem 0;
	}

	.chip {
		background: #333;
		color: #ddd;
		padding: 0.25rem 0.75rem;
		border-radius: 99px;
		font-size: 0.8rem;
	}

	.overview {
		max-width: 600px;
		line-height: 1.6;
	}

	.seasons {
		display: flex;
		gap: 1rem;
		overflow-x: auto;
		padding: 0.5rem 0;
	}

	.season {
		flex-shrink: 0;
		width: 120px;
		text-align: center;
	}

	.season img {
		width: 100%;
		border-radius: 8px;
	}

	.season h3 {
		font-size: 0.85rem;
		margin: 0.25rem 0 0;
	}

	.season p {
		font-size: 0.75rem;
		color: #888;
		margin: 0;
	}

	.trailers {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: 1rem;
	}

	.trailer iframe {
		width: 100%;
		aspect-ratio: 16/9;
		border: none;
		border-radius: 8px;
	}

	.error {
		color: red;
	}
</style>
