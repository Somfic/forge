<script lang="ts">
	import GradientOverlay from "./GradientOverlay.svelte";
	import { onDestroy } from "svelte";

	let {
		images = [],
		interval = 10000,
		overlay = true,
		override,
		position = "center top",
		dominantColor = $bindable("9, 10, 19"),
	}: {
		images: string[];
		interval?: number;
		overlay?: boolean;
		override?: string;
		position?: string;
		dominantColor?: string;
	} = $props();

	let layerAEl: HTMLDivElement;
	let layerBEl: HTMLDivElement;
	let overrideEl: HTMLDivElement;
	let useA = true;
	let index = 0;
	let timer: ReturnType<typeof setInterval> | null = null;
	let imagesKey = "";

	function preloadImg(url: string): Promise<HTMLImageElement> {
		return new Promise((resolve, reject) => {
			const img = new Image();
			img.crossOrigin = "anonymous";
			img.onload = () => resolve(img);
			img.onerror = () => reject();
			img.src = url;
		});
	}

	function extractColor(img: HTMLImageElement): string {
		try {
			const canvas = document.createElement("canvas");
			canvas.width = 32;
			canvas.height = 32;
			const ctx = canvas.getContext("2d");
			if (!ctx) return "9, 10, 19";
			ctx.drawImage(img, 0, 0, 32, 32);
			const data = ctx.getImageData(0, 0, 32, 32).data;
			let r = 0,
				g = 0,
				b = 0,
				count = 0;
			for (let i = 0; i < data.length; i += 4) {
				const br = data[i] + data[i + 1] + data[i + 2];
				if (br > 60 && br < 600) {
					r += data[i];
					g += data[i + 1];
					b += data[i + 2];
					count++;
				}
			}
			if (!count) return "9, 10, 19";
			const d = 0.35;
			return `${Math.round((r / count) * d)}, ${Math.round((g / count) * d)}, ${Math.round((b / count) * d)}`;
		} catch {
			return "9, 10, 19";
		}
	}

	// Direct DOM crossfade — bypasses Svelte batching entirely
	async function crossfadeTo(url: string) {
		if (!layerAEl || !layerBEl) return;
		try {
			const img = await preloadImg(url);
			const color = extractColor(img);
			queueMicrotask(() => {
				dominantColor = color;
			});
		} catch {}

		const incoming = useA ? layerAEl : layerBEl;
		const outgoing = useA ? layerBEl : layerAEl;

		// 1. Set URL on hidden layer
		incoming.style.backgroundImage = `url(${url})`;

		// 2. Force browser to acknowledge the style before transitioning
		incoming.offsetHeight;

		// 3. Fade in new, fade out old
		incoming.style.opacity = "1";
		outgoing.style.opacity = "0";

		useA = !useA;
	}

	function stopCycling() {
		if (timer) {
			clearInterval(timer);
			timer = null;
		}
	}

	function startCycling() {
		stopCycling();
		if (images.length <= 1) return;
		timer = setInterval(() => {
			if (overrideEl?.style.opacity === "1") return;
			index = (index + 1) % images.length;
			crossfadeTo(images[index]);
		}, interval);
	}

	// Watch images
	$effect(() => {
		const key = images.join("\n");
		if (key === imagesKey) return;
		imagesKey = key;
		stopCycling();
		if (!images.length) return;
		index = 0;
		useA = true;
		crossfadeTo(images[0]);
		startCycling();
	});

	// Watch override
	$effect(() => {
		if (!overrideEl) return;
		if (override) {
			preloadImg(override)
				.then((img) => {
					const color = extractColor(img);
					queueMicrotask(() => {
						dominantColor = color;
					});
					overrideEl.style.backgroundImage = `url(${override})`;
					overrideEl.offsetHeight;
					overrideEl.style.opacity = "1";
				})
				.catch(() => {});
		} else {
			overrideEl.style.opacity = "0";
		}
	});

	// Reactively update pan position on all layers
	$effect(() => {
		for (const el of [layerAEl, layerBEl, overrideEl]) {
			if (el) el.style.transform = `translateX(${position})`;
		}
	});

	onDestroy(stopCycling);
</script>

<div class="cycling-backdrop">
	<div class="layer" bind:this={layerAEl}></div>
	<div class="layer" bind:this={layerBEl}></div>
	<div class="layer override" bind:this={overrideEl}></div>
	<GradientOverlay visible={overlay} />
</div>

<style>
	.cycling-backdrop {
		position: absolute;
		inset: 0;
		overflow: hidden;
	}

	.layer {
		position: absolute;
		top: 0;
		left: -15%;
		width: 130%;
		height: 100%;
		background-size: cover;
		background-position: center top;
		opacity: 0;
		transition: opacity 1.5s ease, transform 0.5s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.override {
		z-index: 1;
	}
</style>
