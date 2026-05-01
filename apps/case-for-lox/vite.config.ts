import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, searchForWorkspaceRoot } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
	plugins: [tailwindcss(), wasm(), sveltekit()],
	optimizeDeps: {
		exclude: ['svelte-tweakpane-ui']
	},
	server: {
		fs: {
			allow: [
				// search up for workspace root
				searchForWorkspaceRoot(process.cwd()),
				// your custom rules
				'../../packages/lox-space-wasm'
			]
		}
	}
});
