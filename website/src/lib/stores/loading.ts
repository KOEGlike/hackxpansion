import { writable } from 'svelte/store';
import { asset } from '$app/paths';

export const isLoading = writable(true);
export const loadProgress = writable(0);
export const frameCount = 391;

const extraImages = ['/ferris.png'];
const renderVideos = [
	'/renders/output_h264.mp4',
	'/renders/output_vp9.webm',
	'/renders/output_av1.webm'
];

function loadImage(path: string) {
	return new Promise<void>((resolve) => {
		const img = new Image();
		img.onload = () => resolve();
		img.onerror = () => resolve();
		img.src = asset(path);
	});
}

function loadVideo(path: string) {
	return new Promise<void>((resolve) => {
		const video = document.createElement('video');
		video.preload = 'auto';
		video.onloadeddata = () => resolve();
		video.onerror = () => resolve();
		video.src = asset(path);
		video.load();
	});
}

export async function preloadAssets() {
	const assetsToLoad = [...extraImages, ...renderVideos];
	let loadedCount = 0;

	const promises = assetsToLoad.map((path) => {
		const loader = path.endsWith('.png') ? loadImage(path) : loadVideo(path);

		return loader.finally(() => {
			loadedCount++;
			loadProgress.set((loadedCount / assetsToLoad.length) * 100);
		});
	});

	await Promise.all(promises);
	isLoading.set(false);
}
