export type CanvasResizeDetails = {
	context: CanvasRenderingContext2D;
	viewportWidth: number;
	viewportHeight: number;
	pixelRatio: number;
};

type CanvasResizerOptions = {
	getCanvas: () => HTMLCanvasElement | null;
	onResize?: (details: CanvasResizeDetails) => void;
};

const resizeCanvasToViewport = (
	canvas: HTMLCanvasElement,
	onResize?: (details: CanvasResizeDetails) => void
) => {
	const bounds = canvas.getBoundingClientRect();
	const viewportWidth = Math.max(1, Math.round(bounds.width));
	const viewportHeight = Math.max(1, Math.round(bounds.height));
	const pixelRatio = window.devicePixelRatio || 1;

	canvas.width = Math.floor(viewportWidth * pixelRatio);
	canvas.height = Math.floor(viewportHeight * pixelRatio);

	const context = canvas.getContext('2d');
	if (!context) {
		return;
	}

	context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
	onResize?.({ context, viewportWidth, viewportHeight, pixelRatio });
};

export const createWindowCanvasResizer = ({ getCanvas, onResize }: CanvasResizerOptions) => {
	let resizeRaf = 0;

	const resizeNow = () => {
		const canvas = getCanvas();
		if (!canvas) {
			return;
		}

		resizeCanvasToViewport(canvas, onResize);
	};

	const requestResize = () => {
		if (resizeRaf) {
			cancelAnimationFrame(resizeRaf);
		}

		resizeRaf = requestAnimationFrame(() => {
			resizeRaf = 0;
			resizeNow();
		});
	};

	const start = () => {
		requestResize();
		window.addEventListener('resize', requestResize);
		window.addEventListener('orientationchange', requestResize);
	};

	const stop = () => {
		window.removeEventListener('resize', requestResize);
		window.removeEventListener('orientationchange', requestResize);
		if (resizeRaf) {
			cancelAnimationFrame(resizeRaf);
			resizeRaf = 0;
		}
	};

	return {
		resizeNow,
		requestResize,
		start,
		stop
	};
};
