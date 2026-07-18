import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { enhancedImages } from '@sveltejs/enhanced-img';
import { defineConfig } from 'vite';

const configuredHosts = (process.env.VITE_ALLOWED_HOSTS ?? '')
	.split(',')
	.map((host) => host.trim())
	.filter(Boolean);

export default defineConfig({
	plugins: [tailwindcss(), enhancedImages(), sveltekit()],
	server: {
		allowedHosts: [
			'localhost',
			'127.0.0.1',
			'salad-worry-underpay.ngrok-free.dev',
			...configuredHosts
		]
	}
});
