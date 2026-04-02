<script lang="ts">
	let {
		size = 32,
	}: {
		size?: number;
	} = $props();
</script>

<div class="spinner" style:width="{size}px" style:height="{size}px"></div>

<style>
	.spinner {
		border: 2.5px solid rgba(255, 255, 255, 0.15);
		border-top-color: rgba(255, 255, 255, 0.8);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
