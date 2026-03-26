<script lang="ts">
	let {
		visible = true,
	}: {
		visible?: boolean;
	} = $props();
</script>

<div class="gradient-overlay" class:visible></div>

<style>
	.gradient-overlay {
		position: absolute;
		inset: 0;
		z-index: 2;
		background: linear-gradient(
			rgba(0, 0, 0, 0.3) 0%,
			rgba(9, 10, 19, 0.85) 65%,
			rgb(9, 10, 19) 100%
		);
		pointer-events: none;
		opacity: 0;
		transition: opacity 300ms ease;
	}

	.gradient-overlay.visible {
		opacity: 1;
	}
</style>
