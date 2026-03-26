<script lang="ts">
	import { page } from "$app/state";
	import { goto, replaceState } from "$app/navigation";
	import {
		movieStreams,
		tvStreams,
		movieSubtitles,
		tvSubtitles,
		subtitleCues,
		type MediaItem,
		type MediaType,
		type Stream,
		type SubtitleTrack,
		type SubtitleCue,
	} from "$lib/api.gen";
	import { getDetails, imageUrl, playStream } from "$lib/utils";
	import { Banner, Button, Heading, MediaCard, Pill, Text } from "glow";
	import CyclingBackdrop from "$lib/components/CyclingBackdrop.svelte";
	import GradientOverlay from "$lib/components/GradientOverlay.svelte";
	import VideoPlayer from "$lib/components/VideoPlayer.svelte";

	let item = $state<MediaItem | null>(null);
	let streams = $state<Stream[]>([]);
	let selectedSeason = $state<number | null>(null);
	let selectedEpisode = $state<number | null>(null);
	let loadingStreams = $state(false);
	let error = $state<string | null>(null);
	let backdropColor = $state("9, 10, 19");
	let episodeAreaEl = $state<HTMLDivElement | undefined>(undefined);

	// Player state (page 3)
	let selectedStream = $state<Stream | null>(null);
	let streamUrl = $state<string | null>(null);
	let subtitleTracks = $state<SubtitleTrack[]>([]);
	let activeCues = $state<SubtitleCue[]>([]);
	let activeTrackUrl = $state<string | undefined>(undefined);
	let loadingSubtitles = $state(false);

	// 0 = show info, 1 = season/episodes, 2 = episode detail
	const slideIndex = $derived(
		selectedEpisode !== null ? 2 : selectedSeason !== null ? 1 : 0,
	);

	const backdropUrls = $derived(
		item?.backdrops?.map((b) => imageUrl(b, "original")) ?? [],
	);

	const activeSeason = $derived(
		item?.seasons?.find((s) => s.season_number === selectedSeason) ?? null,
	);

	const activeEpisode = $derived(
		activeSeason?.episodes?.find(
			(e) => e.episode_number === selectedEpisode,
		) ?? null,
	);

	const playerTitle = $derived(
		item?.media_type === "tv" && activeEpisode
			? activeEpisode.name
			: item?.title,
	);

	const playerTopline = $derived(
		item?.media_type === "tv" &&
			selectedSeason !== null &&
			selectedEpisode !== null
			? `S${selectedSeason} E${selectedEpisode} · ${item?.title}`
			: undefined,
	);

	const episodeOverride = $derived(
		activeEpisode?.still_path
			? imageUrl(activeEpisode.still_path, "original")
			: undefined,
	);

	$effect(() => {
		const type = page.params.type as MediaType;
		const id = Number(page.params.id);
		item = null;
		streams = [];
		selectedSeason = page.url.searchParams.has("s")
			? Number(page.url.searchParams.get("s"))
			: null;
		selectedEpisode = page.url.searchParams.has("e")
			? Number(page.url.searchParams.get("e"))
			: null;
		error = null;
		getDetails(type, id)
			.then((res) => {
				item = res.data;
				// Auto-load streams if episode was in URL
				if (selectedSeason !== null && selectedEpisode !== null) {
					loadEpisodeStreams(selectedSeason, selectedEpisode);
				}
			})
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

	function selectSeason(seasonNumber: number) {
		selectedSeason = seasonNumber;
		selectedEpisode = null;
		streams = [];
		updateParams();
		setTimeout(() => scrollToSeason(seasonNumber), 550);
	}

	function selectEpisode(season: number, episode: number) {
		selectedSeason = season;
		selectedEpisode = episode;
		streams = [];
		updateParams();
		loadEpisodeStreams(season, episode);
	}

	function scrollToSeason(seasonNumber: number) {
		selectedSeason = seasonNumber;
		updateParams();
		const el = document.getElementById(`season-${seasonNumber}`);
		if (el && episodeAreaEl) {
			episodeAreaEl.scrollTo({
				top: el.offsetTop - episodeAreaEl.offsetTop,
				behavior: "smooth",
			});
		}
	}

	function goBack() {
		if (selectedStream !== null) {
			stopPlaying();
		} else if (selectedEpisode !== null) {
			selectedEpisode = null;
			streams = [];
		} else {
			selectedSeason = null;
		}
		updateParams();
	}

	function updateParams() {
		const u = new URL(window.location.href);
		if (selectedSeason !== null) {
			u.searchParams.set("s", String(selectedSeason));
		} else {
			u.searchParams.delete("s");
		}
		if (selectedEpisode !== null) {
			u.searchParams.set("e", String(selectedEpisode));
		} else {
			u.searchParams.delete("e");
		}
		replaceState(u, {});
	}

	async function loadEpisodeStreams(season: number, episode: number) {
		if (!item) return;
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
		if (!item) return;
		selectedStream = stream;
		streamUrl = null;
		subtitleTracks = [];
		activeCues = [];
		activeTrackUrl = undefined;

		const u = new URL(window.location.href);
		u.searchParams.set("hash", stream.info_hash);
		u.searchParams.set("file", String(stream.file_idx));
		replaceState(u, {});

		// Start stream + load subtitles in parallel
		playStream(stream.info_hash, stream.file_idx)
			.then((url) => {
				streamUrl = url;
			})
			.catch((e) => {
				error = e.message;
			});

		loadSubtitles();
	}

	async function loadSubtitles() {
		if (!item) return;
		loadingSubtitles = true;
		try {
			if (item.media_type === "movie") {
				const res = await movieSubtitles(item.id);
				subtitleTracks = res.data;
			} else if (selectedSeason !== null && selectedEpisode !== null) {
				const res = await tvSubtitles(
					item.id,
					selectedSeason,
					selectedEpisode,
				);
				subtitleTracks = res.data;
			}
			if (subtitleTracks.length > 0) {
				await selectSubtitleTrack(subtitleTracks[0]);
			}
		} catch {
		} finally {
			loadingSubtitles = false;
		}
	}

	async function selectSubtitleTrack(track: SubtitleTrack) {
		loadingSubtitles = true;
		activeTrackUrl = track.url;
		try {
			const res = await subtitleCues({ url: track.url });
			activeCues = res.data;
		} catch {
			activeCues = [];
		} finally {
			loadingSubtitles = false;
		}
	}

	function disableSubtitles() {
		activeCues = [];
		activeTrackUrl = undefined;
	}

	function stopPlaying() {
		selectedStream = null;
		streamUrl = null;
		subtitleTracks = [];
		activeCues = [];
		activeTrackUrl = undefined;
		const u = new URL(window.location.href);
		u.searchParams.delete("hash");
		u.searchParams.delete("file");
		replaceState(u, {});
	}
</script>

{#if error}
	<Banner variant="error" label={error} />
{:else if !item}
	<Text variant="muted">Loading...</Text>
{:else}
	<!-- Backdrop: 80% on page 0, full on page 1+2 -->
	<div class="backdrop-container">
		<CyclingBackdrop
			images={backdropUrls}
			overlay={slideIndex === 1 || selectedStream !== null}
			override={slideIndex === 2 ? episodeOverride : undefined}
			position={selectedStream
				? "0%"
				: slideIndex === 2
					? "10%"
					: slideIndex === 1
						? "0%"
						: "-10%"}
			bind:dominantColor={backdropColor}
		/>
	</div>
	<div
		class="horizontal-gradient"
		class:hidden={slideIndex > 0 || selectedStream !== null}
		style="--bg-tint: {backdropColor}"
	></div>
	<div
		class="horizontal-gradient-left"
		class:hidden={slideIndex !== 2 || selectedStream !== null}
		style="--bg-tint: {backdropColor}"
	></div>

	<!-- 3-page slider -->
	<div
		class="slider"
		class:faded={selectedStream !== null}
		style="transform: translateX({-slideIndex * 100}vw)"
	>
		<!-- Page 0: Show/Movie info -->
		<div class="page page-info">
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
							>{item.release_date.slice(0, 4)}</Text
						>
					{/if}
					{#if item.runtime}
						<Text as="span" variant="secondary" size="sm"
							>{item.runtime} min</Text
						>
					{/if}
					{#if item.seasons}
						<Text as="span" variant="secondary" size="sm">
							{item.seasons.length} season{item.seasons.length > 1
								? "s"
								: ""}
						</Text>
					{/if}
					{#if item.rating}
						<Text as="span" variant="secondary" size="sm"
							>★ {item.rating.toFixed(1)}</Text
						>
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

				{#if item.media_type === "movie" && streams.length === 0}
					<Button
						variant="primary"
						label={loadingStreams ? "Finding streams..." : "Watch"}
						disabled={loadingStreams}
						loading={loadingStreams}
						onclick={loadMovieStreams}
					/>
				{/if}

				{#if item.media_type === "movie" && streams.length > 0}
					<div class="streams-section">
						<Text weight="semibold" size="sm">Streams</Text>
						{@render streamList()}
					</div>
				{/if}

				{#if item.seasons?.length}
					<div class="season-cards">
						{#each item.seasons as season}
							<MediaCard
								title={season.name}
								src={season.poster_path
									? imageUrl(season.poster_path, "w185")
									: ""}
								aspectRatio="2/3"
								onclick={() =>
									selectSeason(season.season_number)}
							/>
						{/each}
					</div>
				{/if}
			</div>
		</div>

		<!-- Page 1: Season strip + All episodes -->
		<div class="page page-episodes">
			<div class="season-strip">
				<Button variant="ghost" icon="ArrowLeft" onclick={goBack} />
				{#if item.seasons?.length}
					{#each item.seasons as season}
						<MediaCard
							title={`S ${season.season_number}`}
							src={season.poster_path
								? imageUrl(season.poster_path, "w92")
								: ""}
							aspectRatio="2/3"
							onclick={() => scrollToSeason(season.season_number)}
						/>
					{/each}
				{/if}
			</div>

			<div class="episode-area" bind:this={episodeAreaEl}>
				{#if item.seasons?.length}
					{#each item.seasons as season}
						<div
							class="season-section"
							id="season-{season.season_number}"
						>
							<Text weight="semibold" size="sm"
								>{season.name}</Text
							>
							<div class="episode-grid">
								{#each season.episodes as ep}
									<MediaCard
										title={ep.name}
										subtitle={`S${season.season_number} E${ep.episode_number}`}
										src={ep.still_path
											? imageUrl(ep.still_path, "w300")
											: ""}
										aspectRatio="16/9"
										onclick={() =>
											selectEpisode(
												season.season_number,
												ep.episode_number,
											)}
									/>
								{/each}
							</div>
						</div>
					{/each}
				{/if}
			</div>
		</div>

		<!-- Page 2: Episode detail -->
		<div class="page page-episode-detail">
			<!-- Episode strip on left -->
			<div class="episode-strip">
				<Button variant="ghost" icon="ArrowLeft" onclick={goBack} />
				{#if activeSeason}
					{#each activeSeason.episodes as ep}
						<MediaCard
							title="S{activeSeason.season_number} E{ep.episode_number}"
							src={ep.still_path
								? imageUrl(ep.still_path, "w185")
								: ""}
							aspectRatio="16/9"
							onclick={() =>
								selectEpisode(
									activeSeason.season_number,
									ep.episode_number,
								)}
						/>
					{/each}
				{/if}
			</div>

			<!-- Episode info sidebar -->
			{#if activeEpisode}
				<div class="sidebar episode-sidebar">
					<h2 class="ep-title">{activeEpisode.name}</h2>
					<div class="meta">
						<Text as="span" variant="secondary" size="sm">
							S{selectedSeason} E{selectedEpisode}
						</Text>
						<Text as="span" variant="secondary" size="sm"
							>{item.title}</Text
						>
					</div>

					{#if activeEpisode.overview}
						<Text size="sm">{activeEpisode.overview}</Text>
					{/if}

					{#if loadingStreams}
						<Text variant="muted" size="sm">Finding streams...</Text
						>
					{/if}

					{#if streams.length > 0}
						<div class="streams-section">
							<Text weight="semibold" size="sm">Streams</Text>
							{@render streamList()}
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>

	<!-- Player overlay (fades in over everything) -->
	<div class="player-overlay" class:active={selectedStream !== null}>
		{#if selectedStream}
			<VideoPlayer
				src={streamUrl ?? ""}
				subtitles={activeCues}
				title={playerTitle}
				topline={playerTopline}
				titleImage={item?.logo_path
					? imageUrl(item.logo_path, "original")
					: undefined}
				{subtitleTracks}
				{loadingSubtitles}
				{activeTrackUrl}
				onClose={stopPlaying}
				onSubtitleSelect={selectSubtitleTrack}
				onSubtitleOff={disableSubtitles}
				autoplay
			/>
		{/if}
	</div>
{/if}

{#snippet streamList()}
	<div class="streams">
		{#each streams as stream}
			<button class="stream-option" onclick={() => play(stream)}>
				<div class="stream-main">
					{#if stream.resolution}
						<span class="stream-res">{stream.resolution}</span>
					{/if}
					<div class="stream-badges">
						{#if stream.hdr}
							<Pill label="HDR" color="#d97706" />
						{/if}
						{#if stream.imax}
							<Pill label="IMAX" color="#dc2626" />
						{/if}
						{#if stream.codec}
							<Pill label={stream.codec} />
						{/if}
						{#if stream.audio}
							<Pill label={stream.audio} color="#7c3aed" />
						{/if}
						{#if stream.source_type}
							<Pill label={stream.source_type} color="#065f46" />
						{/if}
					</div>
				</div>
				<div class="stream-meta">
					{#if stream.seeders}
						<span class="stream-stat">
							<span class="stream-stat-icon">👤</span>
							{stream.seeders}
						</span>
					{/if}
					{#if stream.size_display}
						<span class="stream-stat">
							<span class="stream-stat-icon">💾</span>
							{stream.size_display}
						</span>
					{/if}
					<span class="stream-source">{stream.source}</span>
				</div>
			</button>
		{/each}
	</div>
{/snippet}

<style>
	/* ── Backdrop ── */
	:global(body) {
		background: transparent !important;
		overflow-x: hidden !important;
	}

	.backdrop-container {
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		z-index: 0;
	}

	.horizontal-gradient {
		position: fixed;
		inset: 0;
		z-index: 0;
		background: linear-gradient(
			to right,
			transparent 0%,
			transparent 40%,
			rgba(var(--bg-tint), 0.6) 55%,
			rgba(var(--bg-tint), 0.9) 70%,
			rgb(var(--bg-tint)) 80%
		);
		pointer-events: none;
		transition: opacity 0.5s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.horizontal-gradient.hidden {
		opacity: 0;
	}

	.horizontal-gradient-left {
		position: fixed;
		inset: 0;
		z-index: 0;
		background: linear-gradient(
			to left,
			transparent 0%,
			transparent 40%,
			rgba(var(--bg-tint), 0.6) 55%,
			rgba(var(--bg-tint), 0.9) 70%,
			rgb(var(--bg-tint)) 80%
		);
		pointer-events: none;
		transition: opacity 0.5s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.horizontal-gradient-left.hidden {
		opacity: 0;
	}

	/* ── Slider ── */
	.slider {
		position: relative;
		z-index: 1;
		display: flex;
		width: 300vw;
		height: 100vh;
		overflow: hidden;
		transition:
			transform 0.5s cubic-bezier(0.4, 0, 0.2, 1),
			opacity 0.5s ease;
	}

	.page {
		width: 100vw;
		height: 100vh;
		flex-shrink: 0;
		overflow-y: auto;
	}

	/* ── Page 0: Info ── */
	.page-info {
		display: flex;
		justify-content: flex-end;
	}

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

	.season-cards {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
		gap: 0.5rem;
		margin-top: auto;
	}

	/* ── Page 1: Episodes ── */
	.page-episodes {
		display: flex;
	}

	.season-strip {
		width: 100px;
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

	.season-strip :global(> *) {
		width: 80px;
		flex-shrink: 0;
	}

	.episode-area {
		flex: 1;
		padding: 2rem;
		padding-top: 2rem;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 2rem;
		scroll-behavior: smooth;
	}

	.season-section {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.episode-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 0.75rem;
	}

	/* ── Page 2: Episode detail ── */
	.page-episode-detail {
		display: flex;
		overflow: hidden;
	}

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

	.episode-sidebar {
		width: 35vw;
		height: 100vh;
		overflow-y: auto;
	}

	.ep-title {
		font-size: 1.5rem;
		font-weight: 700;
		color: white;
		margin: 0;
	}

	/* ── Streams ── */
	.streams-section {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.streams {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}

	.stream-option {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
		background: rgba(255, 255, 255, 0.04);
		backdrop-filter: blur(8px);
		border: 1px solid rgba(255, 255, 255, 0.06);
		border-radius: 10px;
		padding: 0.65rem 0.85rem;
		cursor: pointer;
		text-align: left;
		color: inherit;
		transition:
			background 0.2s,
			border-color 0.2s,
			transform 0.15s;
	}

	.stream-option:hover {
		background: rgba(255, 255, 255, 0.08);
		border-color: rgba(255, 255, 255, 0.12);
		transform: translateY(-1px);
	}

	.stream-option:active {
		transform: translateY(0);
	}

	.stream-main {
		display: flex;
		align-items: center;
		gap: 0.6rem;
	}

	.stream-res {
		font-size: 0.95rem;
		font-weight: 700;
		color: white;
		min-width: 3.5em;
		letter-spacing: -0.01em;
	}

	.stream-badges {
		display: flex;
		gap: 0.25rem;
		flex-wrap: wrap;
		align-items: center;
	}

	.stream-meta {
		display: flex;
		gap: 0.75rem;
		align-items: center;
		padding-left: 4.1em;
	}

	.stream-stat {
		display: flex;
		align-items: center;
		gap: 0.25rem;
		font-size: 0.7rem;
		color: rgba(255, 255, 255, 0.4);
	}

	.stream-stat-icon {
		font-size: 0.65rem;
	}

	.stream-source {
		margin-left: auto;
		font-size: 0.65rem;
		color: rgba(255, 255, 255, 0.2);
		letter-spacing: 0.02em;
	}

	/* ── Player overlay ── */
	.slider.faded {
		opacity: 0;
		pointer-events: none;
	}

	.player-overlay {
		position: fixed;
		inset: 0;
		z-index: 10;
		opacity: 0;
		pointer-events: none;
		transition: opacity 0.5s ease;
	}

	.player-overlay.active {
		opacity: 1;
		pointer-events: auto;
	}

	/* ── Responsive ── */
	@media (max-width: 768px) {
		.sidebar {
			width: 100%;
			padding-top: 30vh;
		}

		.page-info,
		.page-episode-detail {
			justify-content: stretch;
		}
	}
</style>
