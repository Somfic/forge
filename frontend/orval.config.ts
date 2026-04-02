import { defineConfig } from 'orval';

export default defineConfig({
	cinema: {
		input: { target: 'openapi.json' },
		output: {
			target: 'src/lib/api.gen.ts',
			client: 'fetch',
			baseUrl: '/api',
		}
	}
});
