import createClient from 'openapi-fetch';
import type { paths, components } from './api.d';

const client = createClient<paths>({ baseUrl: '/movies/api' });

// Re-export generated types for convenience
export type SearchResult = components['schemas']['SearchResult'];
export type MediaItem = components['schemas']['MediaItem'];
export type MediaType = components['schemas']['MediaType'];
export type Image = components['schemas']['Image'];
export type Season = components['schemas']['Season'];

export function imageUrl(path: string, size: string = 'original'): string {
	return `/movies/api/image/${size}${path}`;
}

export async function search(query: string) {
	const { data, error } = await client.GET('/search', {
		params: { query: { q: query } }
	});
	if (error) throw new Error('Search failed');
	return data;
}

export async function getMovieDetails(id: number) {
	const { data, error } = await client.GET('/movie/{id}', {
		params: { path: { id } }
	});
	if (error) throw new Error('Failed to fetch details');
	return data;
}

export async function getTvDetails(id: number) {
	const { data, error } = await client.GET('/tv/{id}', {
		params: { path: { id } }
	});
	if (error) throw new Error('Failed to fetch details');
	return data;
}

export async function getDetails(type: MediaType, id: number) {
	if (type === 'movie') return getMovieDetails(id);
	return getTvDetails(id);
}
