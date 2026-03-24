<script lang="ts">
	import { page } from "$app/state";
	import { movieStreams, tvStreams, type MediaItem, type MediaType, type Stream } from "$lib/api.gen";
	import { getDetails, imageUrl, playStream } from "$lib/utils";
	import { Media } from "glow";

	let item = $state<MediaItem | null>(null);
	let streams = $state<Stream[]>([]);
	let streamUrl = $state<string | null>(null);
	let selectedSeason = $state<number | null>(null);
	let selectedEpisode = $state<number | null>(null);
	let loadingStreams = $state(false);
	let loadingPlay = $state(false);
	let error = $state<string | null>(null);

	$effect(() => {
		const type = page.params.type as MediaType;
		const id = Number(page.params.id);
		item = null;
		streams = [];
		streamUrl = null;
		selectedSeason = null;
		selectedEpisode = null;
		error = null;
		getDetails(type, id)
			.then((res) => (item = res.data))
			.catch((e) => (error = e.message));
	});

	async function loadMovieStreams() {
		if (!item) return;
		loadingStreams = true;
		try {
			const res = await movieStreams(item.id);
			streams = res.data;
		} catch (e: any) {
			error = e.message;
		} finally {
			loadingStreams = false;
		}
	}

	async function loadEpisodeStreams(season: number, episode: number) {
		if (!item) return;
		selectedSeason = season;
		selectedEpisode = episode;
		streams = [];
		loadingStreams = true;
		try {
			const res = await tvStreams(item.id, season, episode);
			streams = res.data;
		} catch (e: any) {
			error = e.message;
		} finally {
			loadingStreams = false;
		}
	}

	async function play(stream: Stream) {
		loadingPlay = true;
		try {
			const url = await playStream(stream.info_hash, stream.file_idx);
			streamUrl = url;
		} catch (e: any) {
			error = e.message;
		} finally {
			loadingPlay = false;
		}
	}

	function stopPlaying() {
		streamUrl = null;
	}
</script>

{#if error}
	<p class="error">{error}</p>
{:else if !item}
	<p>Loading...</p>
{:else}
	{#if streamUrl}
		<div class="player">
			<div class="player-header">
				<h2>{item.title}</h2>
				<button class="close-btn" onclick={stopPlaying}>Close</button>
			</div>
			<div class="player-video">
				<Media
					src={streamUrl}
					type="video"
					controls
					autoplay
					muted={false}
					fit="contain"
				/>
			</div>
		</div>
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

		{#if item.media_type === "movie"}
			<div class="actions">
				{#if streams.length === 0}
					<button class="watch-btn" onclick={loadMovieStreams} disabled={loadingStreams}>
						{loadingStreams ? "Finding streams..." : "Watch"}
					</button>
				{/if}
			</div>
		{/if}

		{#if streams.length > 0}
			<h2>Streams {#if selectedSeason !== null}(S{selectedSeason}E{selectedEpisode}){/if}</h2>
			<div class="streams">
				{#each streams as stream}
					<button class="stream-option" onclick={() => play(stream)} disabled={loadingPlay}>
						<span class="stream-name">{stream.name}</span>
						<span class="stream-title">{stream.title}</span>
					</button>
				{/each}
			</div>
		{/if}

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
					<div class="season" class:active={selectedSeason === season.season_number}>
						{#if season.poster_path}
							<img
								src={imageUrl(season.poster_path, "w185")}
								alt={season.name}
							/>
						{/if}
						<h3>{season.name}</h3>
						<div class="episodes">
							{#each Array.from({ length: season.episode_count }, (_, i) => i + 1) as ep}
								<button
									class="episode-btn"
									class:active={selectedSeason === season.season_number && selectedEpisode === ep}
									onclick={() => loadEpisodeStreams(season.season_number, ep)}
									disabled={loadingStreams}
								>
									{ep}
								</button>
							{/each}
						</div>
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
{/if}

<style>
	.player {
		width: 100%;
	}

	.player-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.player-video {
		width: 100%;
		aspect-ratio: 16/9;
		border-radius: 12px;
		overflow: hidden;
		background: #000;
	}

	.close-btn {
		background: #333;
		color: #fff;
		border: none;
		padding: 0.5rem 1rem;
		border-radius: 8px;
		cursor: pointer;
	}

	.actions {
		margin: 1rem 0;
	}

	.watch-btn {
		background: #2563eb;
		color: white;
		border: none;
		padding: 0.75rem 2rem;
		border-radius: 8px;
		font-size: 1rem;
		cursor: pointer;
		font-weight: bold;
	}

	.watch-btn:disabled {
		opacity: 0.6;
		cursor: wait;
	}

	.streams {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-bottom: 1rem;
	}

	.stream-option {
		display: flex;
		gap: 1rem;
		align-items: baseline;
		background: #1a1a2e;
		border: 1px solid #333;
		border-radius: 8px;
		padding: 0.75rem 1rem;
		cursor: pointer;
		text-align: left;
		color: inherit;
	}

	.stream-option:hover {
		border-color: #2563eb;
		background: #1e1e3a;
	}

	.stream-name {
		font-weight: bold;
		white-space: pre-line;
		min-width: 80px;
	}

	.stream-title {
		color: #888;
		font-size: 0.85rem;
		white-space: pre-line;
	}

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
		flex-direction: column;
		gap: 1rem;
		padding: 0.5rem 0;
	}

	.season {
		border: 1px solid #333;
		border-radius: 8px;
		padding: 1rem;
	}

	.season img {
		width: 100%;
		border-radius: 8px;
	}

	.season h3 {
		font-size: 0.85rem;
		margin: 0.25rem 0 0;
	}

	.season.active {
		border-color: #2563eb;
	}

	.episodes {
		display: flex;
		flex-wrap: wrap;
		gap: 0.25rem;
		margin-top: 0.5rem;
	}

	.episode-btn {
		width: 2rem;
		height: 2rem;
		border: 1px solid #444;
		border-radius: 4px;
		background: #1a1a2e;
		color: #ccc;
		cursor: pointer;
		font-size: 0.75rem;
	}

	.episode-btn:hover {
		border-color: #2563eb;
		background: #1e1e3a;
	}

	.episode-btn.active {
		background: #2563eb;
		border-color: #2563eb;
		color: white;
	}

	.episode-btn:disabled {
		opacity: 0.5;
		cursor: wait;
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
