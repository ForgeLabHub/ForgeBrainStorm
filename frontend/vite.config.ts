import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5173,
		host: true,
		// Dev server sits behind the runtime's HTTPS proxy, so the proxied
		// hostname must be allowed explicitly (Vite 5.4+ blocks unknown hosts).
		allowedHosts: ['.prod-runtime.all-hands.dev', 'localhost', '127.0.0.1'],
		proxy: {
			'/api': {
				target: 'http://localhost:5000',
				changeOrigin: true
			},
			'/ws': {
				target: 'ws://localhost:5000',
				ws: true
			},
			// Probes live at the root of the API, not under /api: the runtime and
			// deploy/ health checks ask for them there.
			'/health': 'http://localhost:5000',
			'/readyz': 'http://localhost:5000'
		}
	}
});
