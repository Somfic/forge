import { defineConfig } from 'orval';
import { existsSync } from 'fs';

// Use local file if it exists (build.rs generates it), otherwise fetch from server
const specFile = 'openapi.json';
const target = existsSync(specFile) ? specFile : 'http://localhost:3000/movies/api/openapi.json';

export default defineConfig({
	movies: {
		input: { target },
		output: {
			target: 'src/lib/api.gen.ts',
			client: 'fetch',
			baseUrl: '/movies/api',
		}
	}
});
