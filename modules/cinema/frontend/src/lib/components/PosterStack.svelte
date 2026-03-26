<script lang="ts">
	import { Text } from "glow";

	interface PosterItem {
		src: string;
		alt: string;
	}

	let {
		items,
		label,
		onclick,
		width = 120,
		height = 150,
	}: {
		items: PosterItem[];
		label: string;
		onclick?: () => void;
		width?: number;
		height?: number;
	} = $props();

	let hovered = $state(false);

	const posterWidth = 65;
	const count = $derived(items.length);

	const positions = $derived.by(() => {
		if (count <= 1) return items.map(() => 0);
		const targetWidth = hovered ? Math.min(width * 1.3, width + count * 10) : width;
		const step = Math.min(posterWidth - 5, (targetWidth - posterWidth) / (count - 1));
		return items.map((_, i) => i * step);
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="stack"
	class:clickable={!!onclick}
	onclick={onclick}
	onmouseenter={() => (hovered = true)}
	onmouseleave={() => (hovered = false)}
	role={onclick ? "button" : undefined}
	tabindex={onclick ? 0 : undefined}
>
	{#if label}
		<Text size="sm" weight="semibold">{label}</Text>
	{/if}
	<div class="posters" style="width: {width}px; height: {height}px;">
		{#each items as item, i}
			<img
				class="poster"
				style="left: {positions[i]}px; z-index: {count - i};"
				src={item.src}
				alt={item.alt}
			/>
		{/each}
	</div>
</div>

<style>
	.stack {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 0.5rem;
	}

	.stack.clickable {
		cursor: pointer;
	}

	.posters {
		position: relative;
		flex-shrink: 0;
	}

	.poster {
		position: absolute;
		top: 0;
		height: 100%;
		width: 65px;
		object-fit: cover;
		border-radius: 6px;
		box-shadow: 4px 0 12px rgba(0, 0, 0, 0.5);
		transition: left 0.3s cubic-bezier(0.4, 0, 0.2, 1);
	}
</style>
