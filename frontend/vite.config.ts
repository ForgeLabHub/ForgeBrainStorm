import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5173,
		allowedHosts: [".e2b.app"],
		proxy: {
			'/api': {
				target: 'http://localhost:5000',
				changeOrigin: true
			},
			'/ws': {
				target: 'ws://localhost:5000',
				ws: true
			}
		}
	}
});
