<script lang="ts">
	import { onDestroy } from "svelte";
	import { fade } from "svelte/transition";
	// @ts-ignore — hls.js types are resolved at build time
	import Hls from "hls.js";
	import { Button } from "glow";
	import GradientOverlay from "./GradientOverlay.svelte";
	import Spinner from "./Spinner.svelte";

	interface SubtitleCue {
		start: number;
		end: number;
		text: string;
	}

	interface SubtitleTrack {
		id: string;
		language: string;
		url: string;
		score: number;
	}

	let {
		src,
		subtitles = [],
		autoplay = false,
		title,
		topline,
		titleImage,
		subtitleTracks = [],
		onClose,
		onSubtitleSelect,
		onSubtitleOff,
		loadingSubtitles = false,
		activeTrackUrl,
		accent,
		currentTime = $bindable(0),
		duration = $bindable(0),
		paused = $bindable(true),
	}: {
		src: string;
		subtitles?: SubtitleCue[];
		autoplay?: boolean;
		title?: string;
		topline?: string;
		titleImage?: string;
		subtitleTracks?: SubtitleTrack[];
		onClose?: () => void;
		onSubtitleSelect?: (track: SubtitleTrack) => void;
		onSubtitleOff?: () => void;
		loadingSubtitles?: boolean;
		activeTrackUrl?: string;
		accent?: string;
		currentTime?: number;
		duration?: number;
		paused?: boolean;
	} = $props();

	let containerEl = $state<HTMLDivElement | undefined>(undefined);
	let videoEl = $state<HTMLVideoElement | undefined>(undefined);
	let hls: Hls | null = null;

	const defaultOffset = -0.25;

	let buffered = $state(0);
	let volume = $state(1);
	let muted = $state(false);
	let loading = $state(true);
	let streamError = $state<string | null>(null);
	let controlsVisible = $state(true);
	let seeking = $state(false);
	let isFullscreen = $state(false);
	let subtitleMenuOpen = $state(false);
	let audioMenuOpen = $state(false);

	interface AudioTrack {
		id: number;
		name: string;
		lang?: string;
	}
	let audioTracks = $state<AudioTrack[]>([]);
	let activeAudioTrack = $state(0);
	let subtitleOffset = $state(defaultOffset);
	let cursorHidden = $state(false);
	let pausedIdle = $state(false);
	let pauseIdleTimeout: ReturnType<typeof setTimeout>;
	let hideTimeout: ReturnType<typeof setTimeout>;
	let volumeBeforeMute = 1;
	let clickTimeout: ReturnType<typeof setTimeout>;

	const isHls = $derived(src?.includes(".m3u8") || src?.includes("playlist"));

	const activeIndex = $derived.by(() => {
		if (!subtitles?.length) return -1;
		const t = currentTime - subtitleOffset;
		return subtitles.findIndex((s) => t >= s.start && t <= s.end);
	});

	const activeSubtitle = $derived(
		activeIndex >= 0 ? subtitles[activeIndex] : null,
	);

	// Nearby cues for crossfade (prev, current, next)
	const nearbyCues = $derived.by(() => {
		if (!subtitles?.length) return [];
		const center =
			activeIndex >= 0
				? activeIndex
				: subtitles.findIndex((s) => s.start > currentTime);
		const from = Math.max(0, center - 1);
		const to = Math.min(subtitles.length, center + 2);
		return subtitles.slice(from, to);
	});

	const progressPercent = $derived(
		duration > 0 ? (currentTime / duration) * 100 : 0,
	);
	const bufferedPercent = $derived(
		duration > 0 ? (buffered / duration) * 100 : 0,
	);

	function formatTime(seconds: number): string {
		if (!isFinite(seconds) || seconds < 0) return "0:00";
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		const s = Math.floor(seconds % 60);
		if (h > 0)
			return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
		return `${m}:${s.toString().padStart(2, "0")}`;
	}

	function togglePlay() {
		if (!videoEl) return;
		if (videoEl.paused) {
			videoEl.play().catch(() => {});
		} else {
			videoEl.pause();
		}
	}

	function seek(e: MouseEvent & { currentTarget: HTMLDivElement }) {
		if (!videoEl || !duration) return;
		const rect = e.currentTarget.getBoundingClientRect();
		const pct = Math.max(
			0,
			Math.min(1, (e.clientX - rect.left) / rect.width),
		);
		videoEl.currentTime = pct * duration;
	}

	function handleProgressDown(
		e: MouseEvent & { currentTarget: HTMLDivElement },
	) {
		seeking = true;
		seek(e);

		const onMove = (ev: MouseEvent) => {
			if (!videoEl || !duration) return;
			const rect = (
				e.currentTarget as HTMLDivElement
			).getBoundingClientRect();
			const pct = Math.max(
				0,
				Math.min(1, (ev.clientX - rect.left) / rect.width),
			);
			videoEl.currentTime = pct * duration;
		};

		const onUp = () => {
			seeking = false;
			window.removeEventListener("mousemove", onMove);
			window.removeEventListener("mouseup", onUp);
		};

		window.addEventListener("mousemove", onMove);
		window.addEventListener("mouseup", onUp);
	}

	function setVolume(e: Event & { currentTarget: HTMLInputElement }) {
		volume = parseFloat(e.currentTarget.value);
		if (videoEl) videoEl.volume = volume;
		muted = volume === 0;
	}

	function toggleMute() {
		if (muted) {
			volume = volumeBeforeMute || 0.5;
			muted = false;
		} else {
			volumeBeforeMute = volume;
			volume = 0;
			muted = true;
		}
		if (videoEl) {
			videoEl.volume = volume;
			videoEl.muted = muted;
		}
	}

	function toggleFullscreen() {
		if (!containerEl) return;
		if (document.fullscreenElement) {
			document.exitFullscreen();
		} else {
			containerEl.requestFullscreen();
		}
	}

	function showControls() {
		controlsVisible = true;
		cursorHidden = false;
		clearTimeout(hideTimeout);
		if (!paused) {
			hideTimeout = setTimeout(() => {
				controlsVisible = false;
				subtitleMenuOpen = false;
				audioMenuOpen = false;
				cursorHidden = true;
			}, 3000);
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!videoEl) return;
		switch (e.key) {
			case " ":
			case "k":
				e.preventDefault();
				togglePlay();
				break;
			case "f":
				e.preventDefault();
				toggleFullscreen();
				break;
			case "Escape":
				e.preventDefault();
				if (subtitleMenuOpen) {
					subtitleMenuOpen = false;
				} else if (onClose) {
					onClose();
				}
				break;
			case "ArrowLeft":
				e.preventDefault();
				videoEl.currentTime = Math.max(0, videoEl.currentTime - 10);
				showControls();
				break;
			case "ArrowRight":
				e.preventDefault();
				videoEl.currentTime = Math.min(
					duration,
					videoEl.currentTime + 10,
				);
				showControls();
				break;
			case "ArrowUp":
				e.preventDefault();
				volume = Math.min(1, volume + 0.1);
				if (videoEl) videoEl.volume = volume;
				muted = false;
				showControls();
				break;
			case "ArrowDown":
				e.preventDefault();
				volume = Math.max(0, volume - 0.1);
				if (videoEl) videoEl.volume = volume;
				muted = volume === 0;
				showControls();
				break;
			case "m":
				e.preventDefault();
				toggleMute();
				showControls();
				break;
		}
	}

	function initVideo() {
		if (!videoEl || !src) return;
		loading = true;

		if (hls) {
			hls.destroy();
			hls = null;
		}

		if (isHls && Hls.isSupported()) {
			hls = new Hls({
				debug: false,
				enableWorker: true,
				lowLatencyMode: false,
			});
			hls.loadSource(src);
			hls.attachMedia(videoEl);
			hls.on(Hls.Events.MANIFEST_PARSED, () => {
				loading = false;
				if (autoplay) videoEl?.play().catch(() => {});
				// Capture audio tracks
				if (hls && hls.audioTracks.length > 1) {
					audioTracks = hls.audioTracks.map((t: any) => ({
						id: t.id,
						name: t.name || `Track ${t.id + 1}`,
						lang: t.lang,
					}));
					activeAudioTrack = hls.audioTrack;
				}
			});
			hls.on(Hls.Events.ERROR, (_event: any, data: any) => {
				if (data.fatal) {
					loading = false;
					hls?.destroy();
					hls = null;
					streamError = "Stream failed. Try restarting Stremio.";
				}
			});
		} else if (
			isHls &&
			videoEl.canPlayType("application/vnd.apple.mpegurl")
		) {
			videoEl.src = src;
		} else {
			videoEl.src = src;
			if (autoplay) videoEl.play().catch(() => {});
		}
	}

	function handleTimeUpdate() {
		if (!videoEl || seeking) return;
		currentTime = videoEl.currentTime;
		if (videoEl.buffered.length > 0) {
			buffered = videoEl.buffered.end(videoEl.buffered.length - 1);
		}
	}

	function handleFullscreenChange() {
		isFullscreen = !!document.fullscreenElement;
	}

	$effect(() => {
		if (videoEl && src) {
			initVideo();
		}
	});

	$effect(() => {
		clearTimeout(pauseIdleTimeout);
		if (paused && !loading && duration > 0) {
			pauseIdleTimeout = setTimeout(() => {
				pausedIdle = true;
			}, 5000);
		} else {
			pausedIdle = false;
		}
	});

	$effect(() => {
		document.addEventListener("fullscreenchange", handleFullscreenChange);
		return () =>
			document.removeEventListener(
				"fullscreenchange",
				handleFullscreenChange,
			);
	});

	onDestroy(() => {
		if (hls) {
			hls.destroy();
			hls = null;
		}
		clearTimeout(hideTimeout);
		clearTimeout(pauseIdleTimeout);
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="player"
	class:fullscreen={isFullscreen}
	class:cursor-hidden={cursorHidden}
	style:--accent={accent ? `rgb(${accent})` : undefined}
	bind:this={containerEl}
	onmousemove={showControls}
	onmouseenter={showControls}
	onkeydown={handleKeydown}
	tabindex="-1"
>
	<!-- svelte-ignore a11y_media_has_caption -->
	<video
		bind:this={videoEl}
		playsinline
		onclick={() => {
			clearTimeout(clickTimeout);
			clickTimeout = setTimeout(togglePlay, 200);
		}}
		ondblclick={() => {
			clearTimeout(clickTimeout);
			toggleFullscreen();
		}}
		ontimeupdate={handleTimeUpdate}
		onplay={() => {
			paused = false;
			showControls();
		}}
		onpause={() => {
			paused = true;
			controlsVisible = true;
		}}
		onloadedmetadata={() => {
			if (videoEl) duration = videoEl.duration;
		}}
		oncanplay={() => (loading = false)}
		onwaiting={() => (loading = true)}
		onerror={() => {
			streamError = "Stream failed. Try restarting Stremio.";
			loading = false;
		}}
	></video>

	{#if streamError}
		<div class="error-overlay">
			<span class="error-message">{streamError}</span>
			<div class="error-actions">
				<Button
					variant="ghost"
					label="Retry"
					icon="RotateCw"
					onclick={() => {
						streamError = null;
						initVideo();
					}}
				/>
				{#if onClose}
					<Button
						variant="ghost"
						label="Go back"
						icon="ArrowLeft"
						onclick={onClose}
					/>
				{/if}
			</div>
		</div>
	{/if}

	<!-- Gradient overlay when paused but backdrop not yet showing -->
	<GradientOverlay visible={paused && !pausedIdle && !loading} />

	<!-- Single centered title — shown when paused or loading -->
	<div
		class="title-overlay"
		class:visible={loading || (paused && duration > 0)}
	>
		{#if titleImage}
			<img class="title-logo" src={titleImage} alt={title || ""} />
		{:else if title}
			<span class="title-text">{title}</span>
		{/if}
	</div>

	<div class="loading-spinner" class:visible={loading}>
		<Spinner />
	</div>

	<div class="subtitles-container">
		{#each nearbyCues as cue (cue.start)}
			<div
				class="subtitle-line"
				class:active={cue === activeSubtitle && !paused && !loading}
			>
				<p>{@html cue.text}</p>
			</div>
		{/each}
	</div>

	<!-- Top bar: title + back -->
	{#if title || onClose}
		<div class="top-bar" class:visible={controlsVisible || paused}>
			<div class="top-gradient"></div>
			<div class="top-content">
				{#if onClose}
					<Button
						variant="ghost"
						icon="ArrowLeft"
						onclick={onClose}
					/>
				{/if}
				<div class="top-text">
					{#if title}
						<span class="top-title">{title}</span>
					{/if}
					{#if topline}
						<span class="top-topline">{topline}</span>
					{/if}
				</div>
			</div>
		</div>
	{/if}

	<!-- Bottom controls -->
	<div
		class="controls"
		class:visible={controlsVisible || paused}
		class:has-subtitle={!!activeSubtitle}
	>
		<div class="gradient"></div>

		<div
			class="progress-container"
			onmousedown={handleProgressDown}
			role="slider"
			aria-valuenow={currentTime}
			aria-valuemin={0}
			aria-valuemax={duration}
			tabindex="-1"
		>
			<div class="progress-track">
				<div
					class="progress-buffered"
					style="width: {bufferedPercent}%"
				></div>
				<div class="progress-fill" style="width: {progressPercent}%">
					<div class="progress-thumb"></div>
				</div>
			</div>
		</div>

		<div class="controls-bar">
			<div class="controls-left">
				<Button
					variant="ghost"
					icon={paused ? "Play" : "Pause"}
					{loading}
					onclick={togglePlay}
				/>

				<div class="volume-group">
					<Button
						variant="ghost"
						icon={muted || volume === 0
							? "VolumeX"
							: volume < 0.5
								? "Volume1"
								: "Volume2"}
						onclick={toggleMute}
					/>
					<input
						type="range"
						min="0"
						max="1"
						step="0.01"
						value={muted ? 0 : volume}
						oninput={setVolume}
						class="volume-slider"
					/>
				</div>

				<span class="time">
					{formatTime(currentTime)} / {formatTime(duration)}
				</span>
			</div>

			<div class="controls-right">
				{#if audioTracks.length > 1}
					<div class="subtitle-menu-anchor">
						<Button
							variant="ghost"
							icon="AudioLines"
							onclick={() => {
								audioMenuOpen = !audioMenuOpen;
								subtitleMenuOpen = false;
							}}
						/>
						{#if audioMenuOpen}
							<div
								class="subtitle-menu"
								transition:fade={{ duration: 100 }}
							>
								<div class="sub-section">
									<span class="sub-label">Audio</span>
									<div class="sub-options">
										{#each audioTracks as track}
											<button
												class="sub-option"
												class:selected={track.id ===
													activeAudioTrack}
												onclick={() => {
													if (hls)
														hls.audioTrack =
															track.id;
													activeAudioTrack = track.id;
													audioMenuOpen = false;
												}}
											>
												{track.name}{track.lang
													? ` (${track.lang})`
													: ""}
											</button>
										{/each}
									</div>
								</div>
							</div>
						{/if}
					</div>
				{/if}
				{#if subtitleTracks.length > 0}
					<div class="subtitle-menu-anchor">
						<Button
							variant="ghost"
							icon={subtitles.length > 0
								? "Captions"
								: "CaptionsOff"}
							loading={loadingSubtitles}
							onclick={() => {
								subtitleMenuOpen = !subtitleMenuOpen;
								audioMenuOpen = false;
							}}
						/>
						{#if subtitleMenuOpen}
							<div
								class="subtitle-menu"
								transition:fade={{ duration: 100 }}
							>
								<div class="sub-section">
									<span class="sub-label">Language</span>
									<div class="sub-options">
										<button
											class="sub-option"
											class:selected={subtitles.length ===
												0}
											onclick={() => {
												onSubtitleOff?.();
											}}
										>
											Off
										</button>
										{#each subtitleTracks as track, i}
											<button
												class="sub-option"
												class:selected={track.url ===
													activeTrackUrl}
												onclick={() => {
													onSubtitleSelect?.(track);
												}}
											>
												{track.language}{subtitleTracks.filter(
													(t) =>
														t.language ===
														track.language,
												).length > 1
													? ` #${subtitleTracks.filter((t) => t.language === track.language).indexOf(track) + 1}`
													: ""}
											</button>
										{/each}
									</div>
								</div>
								{#if subtitles.length > 0}
									<div class="sub-divider"></div>
									<div class="sub-section">
										<span class="sub-label">Offset</span>
										<div class="sub-offset">
											<Button
												variant="ghost"
												icon="Minus"
												onclick={() => {
													subtitleOffset -= 0.25;
												}}
											/>
											<span class="sub-offset-value"
												>{subtitleOffset -
													defaultOffset >
												0
													? "+"
													: ""}{(
													subtitleOffset -
													defaultOffset
												).toFixed(1)}s</span
											>
											<Button
												variant="ghost"
												icon="Plus"
												onclick={() => {
													subtitleOffset += 0.25;
												}}
											/>
										</div>
									</div>
								{/if}
							</div>
						{/if}
					</div>
				{/if}
				<Button
					variant="ghost"
					icon={isFullscreen ? "Minimize" : "Maximize"}
					onclick={toggleFullscreen}
				/>
			</div>
		</div>
	</div>

	{#if paused && !loading && duration > 0}
		<button
			class="big-play"
			onclick={togglePlay}
			aria-label="Play"
			transition:fade={{ duration: 150 }}
		></button>
	{/if}
</div>

<style>
	.player {
		--player-radius: 0;
		--accent: #e4e4e7;
		--accent-dim: rgba(228, 228, 231, 0.4);
		--surface: rgba(0, 0, 0, 0.75);

		position: relative;
		width: 100%;
		height: 100%;
		background: #000;
		overflow: hidden;
		outline: none;
		user-select: none;
	}

	.player.fullscreen {
		border-radius: 0;
		width: 100vw;
		height: 100vh;
	}

	.player.cursor-hidden,
	.player.cursor-hidden :global(*) {
		cursor: none !important;
	}

	video {
		width: 100%;
		height: 100%;
		object-fit: contain;
		display: block;
		cursor: pointer;
	}

	/* ── Error overlay ── */
	.error-overlay {
		position: absolute;
		inset: 0;
		z-index: 10;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 1rem;
		background: rgba(0, 0, 0, 0.85);
	}

	.error-message {
		color: rgba(255, 255, 255, 0.8);
		font-size: 0.9rem;
	}

	.error-actions {
		display: flex;
		gap: 0.5rem;
	}

	/* ── Title overlay (single element, always mounted) ── */
	.title-overlay {
		position: absolute;
		inset: 0;
		z-index: 3;
		display: flex;
		align-items: center;
		justify-content: center;
		pointer-events: none;
		opacity: 0;
		transition: opacity 150ms ease;
	}

	.title-overlay.visible {
		opacity: 1;
	}

	.title-logo {
		max-width: 600px;
		object-fit: contain;
		filter: drop-shadow(0 0 2px rgba(0, 0, 0, 0.5));
	}

	.title-text {
		color: white;
		font-size: 2rem;
		font-weight: 700;
		text-shadow: 0 2px 12px rgba(0, 0, 0, 0.6);
	}

	/* ── Loading spinner ── */
	.loading-spinner {
		position: absolute;
		top: 75%;
		left: 50%;
		transform: translate(-50%, -50%);
		z-index: 3;
		pointer-events: none;
		opacity: 0;
		transition: opacity 150ms ease;
	}

	.loading-spinner.visible {
		opacity: 1;
	}

	/* ── Big play (invisible click target when paused) ── */
	.big-play {
		position: absolute;
		inset: 0;
		background: none;
		border: none;
		cursor: pointer;
		z-index: 4;
	}

	/* ── Top bar ── */
	.top-bar {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		z-index: 6;
		opacity: 0;
		transition: opacity 0.3s ease;
		pointer-events: none;
	}

	.top-bar.visible {
		opacity: 1;
		pointer-events: auto;
	}

	.top-gradient {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 100px;
		background: linear-gradient(rgba(0, 0, 0, 0.7), transparent);
		pointer-events: none;
	}

	.top-content {
		position: relative;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px;
		z-index: 1;
	}

	.top-text {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.top-topline {
		color: rgba(255, 255, 255, 0.5);
		font-size: 0.7rem;
		font-weight: 500;
		text-shadow: 0 1px 4px rgba(0, 0, 0, 0.6);
		letter-spacing: 0.02em;
	}

	.top-title {
		color: white;
		font-size: 1rem;
		font-weight: 600;
		text-shadow: 0 1px 4px rgba(0, 0, 0, 0.6);
	}

	/* ── Subtitles ── */
	.subtitles-container {
		position: absolute;
		bottom: 120px;
		left: 0;
		right: 0;
		z-index: 4;
		pointer-events: none;
		text-align: center;
	}

	.subtitle-line {
		position: absolute;
		bottom: 0;
		left: 50%;
		transform: translateX(-50%);
		max-width: 80%;
		white-space: pre-wrap;
		opacity: 0;
		transition: opacity 150ms ease;
	}

	.subtitle-line.active {
		opacity: 1;
	}

	.subtitle-line p {
		display: inline;
		font-family: var(--subtitle-font);
		font-size: clamp(2rem, 2.5vw, 3rem);
		font-weight: 500;
		line-height: 1.4;
		color: #fff;
		padding: 0.2em 0.5em;
		border-radius: 4px;
		text-shadow:
			0 1px 3px rgba(0, 0, 0, 0.8),
			0 0 12px rgba(0, 0, 0, 0.4);
		-webkit-box-decoration-break: clone;
		box-decoration-break: clone;
	}

	/* ── Controls ── */
	.controls {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		z-index: 5;
		opacity: 0;
		transition: opacity 0.3s ease;
		pointer-events: none;
	}

	.controls.visible {
		opacity: 1;
		pointer-events: auto;
	}

	.gradient {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 140px;
		background: linear-gradient(transparent, rgba(0, 0, 0, 0.85));
		pointer-events: none;
	}

	/* ── Progress ── */
	.progress-container {
		position: relative;
		height: 20px;
		padding: 7px 0;
		cursor: pointer;
		margin: 0 12px;
		z-index: 1;
	}

	.progress-track {
		position: relative;
		height: 4px;
		background: rgba(255, 255, 255, 0.15);
		border-radius: 2px;
		overflow: visible;
		transition: height 0.15s ease;
	}

	.progress-container:hover .progress-track {
		height: 6px;
	}

	.progress-buffered {
		position: absolute;
		top: 0;
		left: 0;
		height: 100%;
		background: rgba(255, 255, 255, 0.2);
		border-radius: 2px;
		transition: width 0.1s linear;
	}

	.progress-fill {
		position: absolute;
		top: 0;
		left: 0;
		height: 100%;
		background: var(--accent);
		border-radius: 2px;
		display: flex;
		align-items: center;
		justify-content: flex-end;
		transition: width 0.05s linear;
	}

	.progress-thumb {
		width: 14px;
		height: 14px;
		background: var(--accent);
		border-radius: 50%;
		transform: translateX(50%) scale(0);
		transition: transform 0.15s ease;
		flex-shrink: 0;
		box-shadow: 0 0 6px rgba(0, 0, 0, 0.5);
	}

	.progress-container:hover .progress-thumb {
		transform: translateX(50%) scale(1);
	}

	/* ── Controls bar ── */
	.controls-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 12px 10px;
		position: relative;
		z-index: 1;
	}

	.controls-left,
	.controls-right {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	/* ── Volume ── */
	.volume-group {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.volume-slider {
		width: 0;
		opacity: 0;
		transition:
			width 0.2s ease,
			opacity 0.2s ease;
		accent-color: var(--accent);
		height: 4px;
		cursor: pointer;
		appearance: none;
		-webkit-appearance: none;
		background: transparent;
	}

	.volume-group:hover .volume-slider,
	.volume-slider:focus {
		width: 70px;
		opacity: 1;
	}

	.volume-slider::-webkit-slider-runnable-track {
		height: 4px;
		background: rgba(255, 255, 255, 0.2);
		border-radius: 2px;
	}

	.volume-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background: var(--accent);
		margin-top: -4px;
		cursor: pointer;
	}

	.volume-slider::-moz-range-track {
		height: 4px;
		background: rgba(255, 255, 255, 0.2);
		border-radius: 2px;
		border: none;
	}

	.volume-slider::-moz-range-thumb {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background: var(--accent);
		border: none;
		cursor: pointer;
	}

	/* ── Time ── */
	.time {
		font-family: "JetBrains Mono", monospace;
		font-size: 0.75rem;
		font-weight: 400;
		color: var(--accent-dim);
		letter-spacing: 0.02em;
		margin-left: 8px;
		white-space: nowrap;
	}

	/* ── Subtitle menu ── */
	.subtitle-menu-anchor {
		position: relative;
	}

	.subtitle-menu {
		position: absolute;
		bottom: 100%;
		right: 0;
		margin-bottom: 8px;
		background: rgba(20, 20, 28, 0.95);
		backdrop-filter: blur(12px);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 10px;
		padding: 8px;
		min-width: 160px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.sub-section {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.sub-label {
		font-size: 0.65rem;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: rgba(255, 255, 255, 0.35);
		padding: 0 6px;
	}

	.sub-options {
		display: flex;
		flex-direction: column;
	}

	.sub-option {
		display: block;
		width: 100%;
		padding: 5px 10px;
		background: none;
		border: none;
		color: rgba(255, 255, 255, 0.6);
		font-size: 0.8rem;
		text-align: left;
		cursor: pointer;
		border-radius: 6px;
		text-transform: uppercase;
		letter-spacing: 0.03em;
	}

	.sub-option:hover {
		background: rgba(255, 255, 255, 0.08);
		color: #fff;
	}

	.sub-option.selected {
		color: #fff;
		font-weight: 600;
	}

	.sub-divider {
		height: 1px;
		background: rgba(255, 255, 255, 0.08);
		margin: 2px 0;
	}

	.sub-offset {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.sub-offset-value {
		font-family: "JetBrains Mono", monospace;
		font-size: 0.75rem;
		color: rgba(255, 255, 255, 0.7);
		min-width: 3.5em;
		text-align: center;
	}
</style>
