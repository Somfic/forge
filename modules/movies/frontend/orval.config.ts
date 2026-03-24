import { defineConfig } from 'orval';

export default defineConfig({
	movies: {
		input: { target: 'openapi.json' },
		output: {
			target: 'src/lib/api.gen.ts',
			client: 'fetch',
			baseUrl: '/movies/api',
		}
	}
});
