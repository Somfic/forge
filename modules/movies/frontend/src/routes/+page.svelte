<script lang="ts">
	import { search, imageUrl, type SearchResult } from '$lib/api';

	let query = $state('');
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
				results = await search(query);
			} finally {
				loading = false;
			}
		}, 300);
	}
</script>

<h1>Movies & TV</h1>

<input
	type="text"
	placeholder="Search movies and TV shows..."
	bind:value={query}
	oninput={onInput}
/>

{#if loading}
	<p>Searching...</p>
{/if}

{#if results.length > 0}
	<div class="grid">
		{#each results as item}
			<a href="/movies/{item.media_type}/{item.id}" class="card">
				{#if item.poster_path}
					<img src={imageUrl(item.poster_path, 'w342')} alt={item.title} />
				{:else}
					<div class="no-poster">{item.title}</div>
				{/if}
				<h3>{item.title}</h3>
				<div class="meta">
					<span class="badge" class:tv={item.media_type === 'tv'}>
						{item.media_type === 'movie' ? 'Movie' : 'TV'}
					</span>
					{#if item.release_date}
						<span>{item.release_date.slice(0, 4)}</span>
					{/if}
				</div>
			</a>
		{/each}
	</div>
{/if}

<style>
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		gap: 1rem;
		margin-top: 1rem;
	}

	.card {
		text-decoration: none;
		color: inherit;
	}

	.card img {
		width: 100%;
		border-radius: 8px;
	}

	.no-poster {
		width: 100%;
		aspect-ratio: 2/3;
		background: #333;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #999;
		padding: 1rem;
		text-align: center;
	}

	input {
		width: 100%;
		max-width: 400px;
		padding: 0.5rem 1rem;
		font-size: 1rem;
		border: 1px solid #ccc;
		border-radius: 8px;
	}

	h3 {
		margin: 0.5rem 0 0;
		font-size: 0.9rem;
	}

	.meta {
		display: flex;
		gap: 0.5rem;
		align-items: center;
		margin-top: 0.25rem;
		color: #888;
		font-size: 0.8rem;
	}

	.badge {
		background: #2563eb;
		color: white;
		padding: 0.1rem 0.4rem;
		border-radius: 4px;
		font-size: 0.7rem;
		font-weight: bold;
	}

	.badge.tv {
		background: #7c3aed;
	}
</style>
