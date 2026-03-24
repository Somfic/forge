const API_BASE = '/movies/api';

export type MediaType = 'movie' | 'tv';

export interface SearchResult {
	id: number;
	media_type: MediaType;
	title: string;
	overview: string | null;
	release_date: string | null;
	poster_path: string | null;
	backdrop_path: string | null;
}

export interface MediaItem {
	id: number;
	media_type: MediaType;
	title: string;
	overview: string | null;
	tagline: string | null;
	release_date: string | null;
	runtime: number | null;
	rating: number | null;
	poster_path: string | null;
	backdrop_path: string | null;
	genres: { id: number; name: string }[];
	videos: { key: string; site: string; name: string; video_type: string }[];
	images: {
		posters: Image[];
		backdrops: Image[];
		logos: Image[];
	} | null;
	seasons: Season[] | null;
}

export interface Image {
	file_path: string;
	width: number;
	height: number;
	iso_639_1: string | null;
	vote_average: number;
}

export interface Season {
	id: number;
	season_number: number;
	name: string;
	episode_count: number;
	poster_path: string | null;
	air_date: string | null;
}

export function imageUrl(path: string, size: string = 'original'): string {
	return `${API_BASE}/image/${size}${path}`;
}

export async function search(query: string): Promise<SearchResult[]> {
	const res = await fetch(`${API_BASE}/search?q=${encodeURIComponent(query)}`);
	if (!res.ok) throw new Error(`Search failed: ${res.statusText}`);
	return res.json();
}

export async function getDetails(type: MediaType, id: number): Promise<MediaItem> {
	const res = await fetch(`${API_BASE}/${type}/${id}`);
	if (!res.ok) throw new Error(`Failed to fetch details: ${res.statusText}`);
	return res.json();
}
