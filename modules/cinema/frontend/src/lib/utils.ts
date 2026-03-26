import { movieDetails, tvDetails, startStream, type MediaType } from './api.gen';

export function imageUrl(path: string, size: string = 'original'): string {
	return `/cinema/api/image/${size}${path}`;
}

export async function getDetails(type: MediaType, id: number) {
	if (type === 'movie') return movieDetails(id);
	return tvDetails(id);
}

export async function playStream(infoHash: string, fileIdx: number) {
	const res = await startStream(infoHash, fileIdx);
	return res.data.url;
}
