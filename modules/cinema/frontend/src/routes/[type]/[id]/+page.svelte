<script lang="ts">
	import { page } from "$app/state";
	import { replaceState } from "$app/navigation";
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
	import { Banner, Text } from "glow";
	import CyclingBackdrop from "$lib/components/CyclingBackdrop.svelte";
	import VideoPlayer from "$lib/components/VideoPlayer.svelte";
	import MediaInfo from "$lib/components/MediaInfo.svelte";
	import SeasonBrowser from "$lib/components/SeasonBrowser.svelte";
	import EpisodeDetail from "$lib/components/EpisodeDetail.svelte";

	// ── Core state ──
	let item = $state<MediaItem | null>(null);
	let streams = $state<Stream[]>([]);
	let selectedSeason = $state<number | null>(null);
	let selectedEpisode = $state<number | null>(null);
	let loadingStreams = $state(false);
	let error = $state<string | null>(null);
	let backdropColor = $state("9, 10, 19");
	let accentColor = $state("228, 228, 231");

	// ── Player state ──
	let selectedStream = $state<Stream | null>(null);
	let streamUrl = $state<string | null>(null);
	let subtitleTracks = $state<SubtitleTrack[]>([]);
	let activeCues = $state<SubtitleCue[]>([]);
	let activeTrackUrl = $state<string | undefined>(undefined);
	let loadingSubtitles = $state(false);

	// ── Derived ──
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
		activeSeason?.episodes?.find((e) => e.episode_number === selectedEpisode) ?? null,
	);

	const playerTitle = $derived(
		item?.media_type === "tv" && activeEpisode ? activeEpisode.name : item?.title,
	);

	const playerTopline = $derived(
		item?.media_type === "tv" && selectedSeason !== null && selectedEpisode !== null
			? `S${selectedSeason} E${selectedEpisode} · ${item?.title}`
			: undefined,
	);

	const episodeOverride = $derived(
		activeEpisode?.still_path ? imageUrl(activeEpisode.still_path, "original") : undefined,
	);

	const backdropPosition = $derived(
		selectedStream ? "0%" : slideIndex === 2 ? "10%" : slideIndex === 1 ? "0%" : "-10%",
	);

	// ── Gradient color transition via @property ──
	let gradientRightEl = $state<HTMLDivElement>(undefined!);
	let gradientLeftEl = $state<HTMLDivElement>(undefined!);

	$effect(() => {
		const [r, g, b] = backdropColor.split(",").map((s) => s.trim());
		for (const el of [gradientRightEl, gradientLeftEl]) {
			if (!el) continue;
			el.style.setProperty("--tint-r", r);
			el.style.setProperty("--tint-g", g);
			el.style.setProperty("--tint-b", b);
		}
	});

	// ── Body style management ──
	$effect(() => {
		document.body.style.overflowX = selectedStream ? "" : "hidden";
		if (selectedStream) {
			document.body.style.setProperty("display", "block", "important");
		} else {
			document.body.style.removeProperty("display");
		}
		return () => {
			document.body.style.overflowX = "";
			document.body.style.removeProperty("display");
		};
	});

	// ── Data loading ──
	$effect(() => {
		const type = page.params.type as MediaType;
		const id = Number(page.params.id);
		item = null;
		streams = [];
		selectedSeason = page.url.searchParams.has("s") ? Number(page.url.searchParams.get("s")) : null;
		selectedEpisode = page.url.searchParams.has("e") ? Number(page.url.searchParams.get("e")) : null;
		error = null;
		getDetails(type, id)
			.then((res) => {
				item = res.data;
				if (selectedSeason !== null && selectedEpisode !== null) {
					loadEpisodeStreams(selectedSeason, selectedEpisode);
				}
			})
			.catch((e) => (error = e.message));
	});

	// ── Navigation ──
	function selectSeason(seasonNumber: number) {
		selectedSeason = seasonNumber;
		selectedEpisode = null;
		streams = [];
		updateParams();
	}

	function selectEpisode(season: number, episode: number) {
		selectedSeason = season;
		selectedEpisode = episode;
		streams = [];
		updateParams();
		loadEpisodeStreams(season, episode);
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
		if (selectedSeason !== null) u.searchParams.set("s", String(selectedSeason));
		else u.searchParams.delete("s");
		if (selectedEpisode !== null) u.searchParams.set("e", String(selectedEpisode));
		else u.searchParams.delete("e");
		replaceState(u, {});
	}

	// ── Stream loading ──
	async function loadMovieStreams() {
		if (!item) return;
		loadingStreams = true;
		try {
			streams = (await movieStreams(item.id)).data;
		} catch (e: any) { error = e.message; }
		finally { loadingStreams = false; }
	}

	async function loadEpisodeStreams(season: number, episode: number) {
		if (!item) return;
		loadingStreams = true;
		try {
			streams = (await tvStreams(item.id, season, episode)).data;
		} catch (e: any) { error = e.message; }
		finally { loadingStreams = false; }
	}

	// ── Player ──
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

		playStream(stream.info_hash, stream.file_idx)
			.then((url) => { streamUrl = url; })
			.catch((e) => { error = e.message; });
		loadSubtitles();
	}

	async function loadSubtitles() {
		if (!item) return;
		loadingSubtitles = true;
		try {
			if (item.media_type === "movie") {
				subtitleTracks = (await movieSubtitles(item.id)).data;
			} else if (selectedSeason !== null && selectedEpisode !== null) {
				subtitleTracks = (await tvSubtitles(item.id, selectedSeason, selectedEpisode)).data;
			}
			if (subtitleTracks.length > 0) await selectSubtitleTrack(subtitleTracks[0]);
		} catch {} finally { loadingSubtitles = false; }
	}

	async function selectSubtitleTrack(track: SubtitleTrack) {
		loadingSubtitles = true;
		activeTrackUrl = track.url;
		try { activeCues = (await subtitleCues({ url: track.url })).data; }
		catch { activeCues = []; }
		finally { loadingSubtitles = false; }
	}

	function disableSubtitles() { activeCues = []; activeTrackUrl = undefined; }

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
	<!-- Backdrop -->
	<div class="backdrop-container">
		<CyclingBackdrop
			images={backdropUrls}
			overlay={slideIndex === 1 || selectedStream !== null}
			override={selectedStream ? backdropUrls[0] : slideIndex === 2 ? episodeOverride : undefined}
			position={backdropPosition}
			bind:dominantColor={backdropColor}
			bind:accentColor
		/>
	</div>
	<div class="gradient-right" class:hidden={slideIndex > 0 || selectedStream !== null} bind:this={gradientRightEl}></div>
	<div class="gradient-left" class:hidden={slideIndex !== 2 || selectedStream !== null} bind:this={gradientLeftEl}></div>

	<!-- Slider -->
	<div class="slider" class:faded={selectedStream !== null} style="transform: translateX({-slideIndex * 100}vw)">
		<!-- Page 0: Info -->
		<div class="page page-info">
			<MediaInfo
				{item}
				{streams}
				{loadingStreams}
				onwatch={loadMovieStreams}
				onplay={play}
				onselectseason={selectSeason}
			/>
		</div>

		<!-- Page 1: Seasons + Episodes -->
		<div class="page page-episodes">
			{#if item.seasons?.length}
				<SeasonBrowser
					seasons={item.seasons}
					onback={goBack}
					onscrollseason={(n) => { selectedSeason = n; updateParams(); }}
					onselectepisode={selectEpisode}
				/>
			{/if}
		</div>

		<!-- Page 2: Episode Detail -->
		<div class="page page-episode-detail">
			{#if activeSeason && activeEpisode}
				<EpisodeDetail
					season={activeSeason}
					episode={activeEpisode}
					showTitle={item.title}
					{streams}
					{loadingStreams}
					onback={goBack}
					onselectepisode={selectEpisode}
					onplay={play}
				/>
			{/if}
		</div>
	</div>

	<!-- Player overlay -->
	<div class="player-overlay" class:active={selectedStream !== null}>
		{#if selectedStream}
			<VideoPlayer
				src={streamUrl ?? ""}
				subtitles={activeCues}
				title={playerTitle}
				topline={playerTopline}
				titleImage={item?.logo_path ? imageUrl(item.logo_path, "original") : undefined}
				{subtitleTracks}
				{loadingSubtitles}
				{activeTrackUrl}
				accent={accentColor}
				onClose={stopPlaying}
				onSubtitleSelect={selectSubtitleTrack}
				onSubtitleOff={disableSubtitles}
				autoplay
			/>
		{/if}
	</div>
{/if}

<style>
	:global(body) {
		background: transparent !important;
	}

	/* ── Backdrop ── */
	.backdrop-container {
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		z-index: 0;
	}

	@property --tint-r { syntax: "<number>"; inherits: false; initial-value: 9; }
	@property --tint-g { syntax: "<number>"; inherits: false; initial-value: 10; }
	@property --tint-b { syntax: "<number>"; inherits: false; initial-value: 19; }

	.gradient-right, .gradient-left {
		position: fixed;
		inset: 0;
		z-index: 0;
		pointer-events: none;
		--tint-r: 9;
		--tint-g: 10;
		--tint-b: 19;
		transition: --tint-r 1.5s ease, --tint-g 1.5s ease, --tint-b 1.5s ease, opacity 0.5s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.gradient-right {
		background: linear-gradient(
			to right,
			transparent 0%,
			transparent 40%,
			rgba(var(--tint-r), var(--tint-g), var(--tint-b), 0.6) 55%,
			rgba(var(--tint-r), var(--tint-g), var(--tint-b), 0.9) 70%,
			rgb(var(--tint-r), var(--tint-g), var(--tint-b)) 80%
		);
	}

	.gradient-left {
		background: linear-gradient(
			to left,
			transparent 0%,
			transparent 40%,
			rgba(var(--tint-r), var(--tint-g), var(--tint-b), 0.6) 55%,
			rgba(var(--tint-r), var(--tint-g), var(--tint-b), 0.9) 70%,
			rgb(var(--tint-r), var(--tint-g), var(--tint-b)) 80%
		);
	}

	.gradient-right.hidden, .gradient-left.hidden {
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
		transition: transform 0.5s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.5s ease;
	}

	.slider.faded {
		opacity: 0;
		pointer-events: none;
	}

	.page {
		width: 100vw;
		height: 100vh;
		flex-shrink: 0;
		overflow-y: auto;
	}

	.page-info {
		display: flex;
		justify-content: flex-end;
	}

	.page-episodes {
		display: flex;
	}

	.page-episode-detail {
		display: flex;
		overflow: hidden;
	}

	/* ── Player ── */
	.player-overlay {
		position: fixed;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
		z-index: 10;
		opacity: 0;
		pointer-events: none;
		transition: opacity 0.5s ease;
	}

	.player-overlay.active {
		opacity: 1;
		pointer-events: auto;
	}
</style>
