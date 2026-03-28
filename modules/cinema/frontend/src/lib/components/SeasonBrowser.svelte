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

	const selectableSeasons = $derived(seasons.filter((s) => s.episodes.length > 0));
	const cols = $derived(selectableSeasons.length <= 4 ? selectableSeasons.length : Math.ceil(selectableSeasons.length / 2));
</script>

<div class="browser">
	<div class="back">
		<Button variant="ghost" icon="ArrowLeft" onclick={onback} />
	</div>
	<div class="season-grid" style="grid-template-columns: repeat({cols}, 180px)">
		{#each selectableSeasons as season}
			<MediaCard
				src={season.poster_path
					? imageUrl(season.poster_path, "w780")
					: ""}
				aspectRatio="2/3"
				onclick={() => {
					onscrollseason(season.season_number);
					onselectepisode(season.season_number, 1);
				}}
			/>
		{/each}
	</div>
</div>

<style>
	.browser {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
		padding: 2rem;
		gap: 1rem;
		background: rgba(0, 0, 0, 0.35);
		backdrop-filter: blur(16px);
	}

	.back {
		position: absolute;
		top: 2rem;
		left: 2rem;
	}

	.season-grid {
		display: grid;
		gap: 1rem;
		justify-content: center;
		overflow-y: auto;
		width: 100%;
	}

	@media (max-width: 768px) {
		.browser {
			padding: 1rem;
		}

		.season-grid {
			grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
		}
	}
</style>
