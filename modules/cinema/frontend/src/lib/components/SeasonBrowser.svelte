<script lang="ts">
	import type { Season } from "$lib/api.gen";
	import { imageUrl } from "$lib/utils";
	import { Button, MediaCard, Text } from "glow";

	let {
		seasons,
		onback,
		onscrollseason,
		onselectepisode,
	}: {
		seasons: Season[];
		onback: () => void;
		onscrollseason: (seasonNumber: number) => void;
		onselectepisode: (season: number, episode: number) => void;
	} = $props();

	let episodeAreaEl = $state<HTMLDivElement | undefined>(undefined);

	export function scrollToSeason(seasonNumber: number) {
		const el = document.getElementById(`season-${seasonNumber}`);
		if (el && episodeAreaEl) {
			episodeAreaEl.scrollTo({
				top: el.offsetTop - episodeAreaEl.offsetTop,
				behavior: "smooth",
			});
		}
	}
</script>

<div class="season-strip">
	<Button variant="ghost" icon="ArrowLeft" onclick={onback} />
	{#each seasons as season}
		<MediaCard
			title={`S ${season.season_number}`}
			src={season.poster_path ? imageUrl(season.poster_path, "w92") : ""}
			aspectRatio="2/3"
			onclick={() => {
				onscrollseason(season.season_number);
				scrollToSeason(season.season_number);
			}}
		/>
	{/each}
</div>

<div class="episode-area" bind:this={episodeAreaEl}>
	{#each seasons as season}
		<div class="season-section" id="season-{season.season_number}">
			<Text weight="semibold" size="sm">{season.name}</Text>
			<div class="episode-grid">
				{#each season.episodes as ep}
					<MediaCard
						title={ep.name}
						subtitle={`S${season.season_number} E${ep.episode_number}`}
						src={ep.still_path ? imageUrl(ep.still_path) : ""}
						aspectRatio="16/9"
						onclick={() =>
							onselectepisode(
								season.season_number,
								ep.episode_number,
							)}
					/>
				{/each}
			</div>
		</div>
	{/each}
</div>

<style>
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
</style>
