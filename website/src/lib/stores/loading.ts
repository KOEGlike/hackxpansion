import { writable } from 'svelte/store';
import { asset } from '$app/paths';

export const isLoading = writable(true);
export const loadProgress = writable(0);
export const imageCount = 157;

export async function preloadImages() {
	const imageDir = '/renders/';
	let loadedCount = 0;

	const promises = Array.from({ length: imageCount }, (_, i) => {
		const filename = String(i).padStart(4, '0') + '.webp';
		return new Promise<void>((resolve) => {
			const img = new Image();
			img.onload = () => {
				loadedCount++;
				loadProgress.set((loadedCount / imageCount) * 100);
				resolve();
			};
			img.onerror = () => {
				loadedCount++;
				loadProgress.set((loadedCount / imageCount) * 100);
				resolve();
			};
			img.src = asset(`${imageDir}${filename}`);
		});
	});

	await Promise.all(promises);
	isLoading.set(false);
}
