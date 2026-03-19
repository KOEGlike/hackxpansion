import { writable } from 'svelte/store';
import { asset } from '$app/paths';

export const isLoading = writable(true);
export const loadProgress = writable(0);
export const imageCount = 390;
export const preloadedFrames = writable<(HTMLImageElement | null)[]>([]);

const extraImages = [asset('/ferris.webp')];
let preloadPromise: Promise<void> | null = null;
let hasFinishedPreload = false;

export async function preloadImages() {
	if (hasFinishedPreload) {
		return;
	}

	if (preloadPromise) {
		await preloadPromise;
		return;
	}

	preloadPromise = preloadAllImages();
	await preloadPromise;
}

async function preloadAllImages() {
	const imageDir = '/renders/';
	const renderImages = Array.from(
		{ length: imageCount },
		(_, i) => `${imageDir}${String(i).padStart(4, '0')}.webp`
	);
	const frameImages: (HTMLImageElement | null)[] = Array.from({ length: imageCount }, () => null);
	const allImages = [...renderImages, ...extraImages];
	let loadedCount = 0;

	const updateProgress = () => {
		loadProgress.set((loadedCount / allImages.length) * 100);
	};

	const renderPromises = renderImages.map((path, index) => {
		return new Promise<void>((resolve) => {
			const img = new Image();
			img.onload = () => {
				frameImages[index] = img;
				loadedCount++;
				updateProgress();
				resolve();
			};
			img.onerror = () => {
				loadedCount++;
				updateProgress();
				resolve();
			};
			img.src = asset(path);
		});
	});

	const extraPromises = extraImages.map((path) => {
		return new Promise<void>((resolve) => {
			const img = new Image();
			img.onload = () => {
				loadedCount++;
				updateProgress();
				resolve();
			};
			img.onerror = () => {
				loadedCount++;
				updateProgress();
				resolve();
			};
			img.src = asset(path);
		});
	});

	await Promise.all([...renderPromises, ...extraPromises]);
	preloadedFrames.set(frameImages);
	hasFinishedPreload = true;
	isLoading.set(false);
}
