export {
	search,
	movieDetails,
	tvDetails,
	type SearchResult,
	type MediaItem,
	type MediaType,
	type Image,
	type Season,
	type SearchParams,
} from './api.gen';

import { movieDetails, tvDetails, type MediaType } from './api.gen';

export function imageUrl(path: string, size: string = 'original'): string {
	return `/movies/api/image/${size}${path}`;
}

export async function getDetails(type: MediaType, id: number) {
	if (type === 'movie') return movieDetails(id);
	return tvDetails(id);
}
