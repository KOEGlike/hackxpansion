import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { enhancedImages } from '@sveltejs/enhanced-img';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [tailwindcss(), enhancedImages(), sveltekit()],
  server: {
    allowedHosts: ['localhost', '127.0.0.1', 'salad-worry-underpay.ngrok-free.dev'],
  },
});
