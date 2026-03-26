<script lang="ts">
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import {
		movieSubtitles, tvSubtitles, subtitleCues,
		type MediaItem, type MediaType, type SubtitleTrack, type SubtitleCue
	} from "$lib/api.gen";
	import { getDetails, imageUrl, playStream } from "$lib/utils";
	import VideoPlayer from "$lib/components/VideoPlayer.svelte";

	let item = $state<MediaItem | null>(null);
	let streamUrl = $state<string | null>(null);
	let subtitleTracks = $state<SubtitleTrack[]>([]);
	let activeCues = $state<SubtitleCue[]>([]);
	let activeTrackUrl = $state<string | undefined>(undefined);
	let loadingSubtitles = $state(false);
	let error = $state<string | null>(null);

	const backdropUrls = $derived(
		item?.backdrops?.map((b) => imageUrl(b, 'original')) ?? []
	);

	const mediaType = $derived(page.params.type as MediaType);
	const mediaId = $derived(Number(page.params.id));
	const infoHash = $derived(page.params.infoHash);
	const fileIdx = $derived(Number(page.params.fileIdx));

	// Parse season/episode from query params for TV
	const season = $derived(page.url.searchParams.get('s') ? Number(page.url.searchParams.get('s')) : null);
	const episode = $derived(page.url.searchParams.get('e') ? Number(page.url.searchParams.get('e')) : null);

	const episodeName = $derived(() => {
		if (!item || mediaType !== 'tv' || season === null || episode === null) return null;
		const s = item.seasons?.find((s) => s.season_number === season);
		return s?.episodes?.find((e) => e.episode_number === episode)?.name ?? null;
	});

	const playerTitle = $derived(
		mediaType === 'tv' && item
			? episodeName() ?? `Episode ${episode}`
			: item?.title
	);
	const playerTopline = $derived(
		mediaType === 'tv' && item && season !== null && episode !== null
			? `S${season} E${episode} · ${item.title}`
			: undefined
	);

	$effect(() => {
		// Load media details + start stream in parallel
		const detailsPromise = getDetails(mediaType, mediaId)
			.then((res) => { item = res.data; })
			.catch((e) => { error = e.message; });

		const streamPromise = playStream(infoHash as string, fileIdx as number)
			.then((url) => { streamUrl = url; })
			.catch((e) => { error = e.message; });

		Promise.all([detailsPromise, streamPromise]).then(() => {
			loadSubtitleTracks();
		});
	});

	async function loadSubtitleTracks() {
		if (!item) return;
		loadingSubtitles = true;
		try {
			if (mediaType === 'movie') {
				const res = await movieSubtitles(item.id);
				subtitleTracks = res.data;
			} else if (season !== null && episode !== null) {
				const res = await tvSubtitles(item.id, season, episode);
				subtitleTracks = res.data;
			}

			if (subtitleTracks.length > 0) {
				await selectSubtitleTrack(subtitleTracks[0]);
			}
		} catch {
			// Subtitles are optional
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

	function close() {
		goto(`/cinema/${mediaType}/${mediaId}`);
	}
</script>

<div class="player-page">
	{#if error}
		<div class="error-overlay">
			<p>{error}</p>
			<button onclick={close}>Go back</button>
		</div>
	{:else if streamUrl}
		<VideoPlayer
			src={streamUrl}
			subtitles={activeCues}
			title={playerTitle}
			topline={playerTopline}
			titleImage={item?.logo_path ? imageUrl(item.logo_path, 'original') : undefined}

			{subtitleTracks}
			{loadingSubtitles}
			{activeTrackUrl}
			onClose={close}
			onSubtitleSelect={selectSubtitleTrack}
			onSubtitleOff={disableSubtitles}
			autoplay
		/>
	{:else}
		<VideoPlayer
			src=""
			title={playerTitle}
			topline={playerTopline}
			titleImage={item?.logo_path ? imageUrl(item.logo_path, 'original') : undefined}

			onClose={close}
		/>
	{/if}
</div>

<style>
	.player-page {
		position: fixed;
		inset: 0;
		z-index: 100;
		background: #000;
	}

	.error-overlay {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: white;
		gap: 1rem;
	}

	.error-overlay button {
		padding: 0.5rem 1.5rem;
		background: rgba(255, 255, 255, 0.1);
		border: 1px solid rgba(255, 255, 255, 0.2);
		color: white;
		border-radius: 8px;
		cursor: pointer;
	}
</style>
