<script lang="ts">
	import "glow/styles";

	let { children } = $props();
</script>

{@render children()}

<style>
	@font-face {
		font-family: 'Subtitle';
		src: url('/cinema/fonts/Helvetica Neue 67 Medium Condensed.otf') format('opentype');
		font-display: swap;
	}

	:global(:root) {
		--subtitle-font: 'Subtitle', system-ui, sans-serif;
	}
</style>
