import { writable } from 'svelte/store';
import { asset } from '$app/paths';

export const isLoading = writable(true);
export const loadProgress = writable(0);
export const imageCount = 157;

const extraImages = ['ferris.png'];

export async function preloadImages() {
	const imageDir = '/renders/';
	const renderImages = Array.from(
		{ length: imageCount },
		(_, i) => `${imageDir}${String(i).padStart(4, '0')}.webp`
	);
	const allImages = [...renderImages, ...extraImages];
	let loadedCount = 0;

	const promises = allImages.map((path) => {
		return new Promise<void>((resolve) => {
			const img = new Image();
			img.onload = () => {
				loadedCount++;
				loadProgress.set((loadedCount / allImages.length) * 100);
				resolve();
			};
			img.onerror = () => {
				loadedCount++;
				loadProgress.set((loadedCount / allImages.length) * 100);
				resolve();
			};
			img.src = asset(path);
		});
	});

	await Promise.all(promises);
	isLoading.set(false);
}
