import { goto } from '$app/navigation';
import type { ClientMsg } from './ws-client-msg.gen';
import type { ServerMsg, Role } from './ws-server-msg.gen';

export type { ClientMsg, ServerMsg, Role };

export type Phase = 'connecting' | 'lobby' | 'picking' | 'ready' | 'watching' | 'disconnected';

export interface ContentPick {
	media_type: string;
	tmdb_id: number;
	title: string;
	poster_path?: string | null;
	info_hash: string;
	file_idx: number;
	season?: number | null;
	episode?: number | null;
}

export interface SyncState {
	playing: boolean;
	position: number;
	serverTime: number;
}

export interface WatchPartyState {
	active: boolean;
	phase: Phase;
	roomCode: string | null;
	myId: number | null;
	role: Role | null;
	participantCount: number;
	content: ContentPick | null;
	readyIds: number[];
	readyTotal: number;
	sync: SyncState | null;
	error: string | null;
}

// ── Module-level singleton ──

let ws: WebSocket | null = null;

export const party: WatchPartyState = $state({
	active: false,
	phase: 'disconnected',
	roomCode: null,
	myId: null,
	role: null,
	participantCount: 0,
	content: null,
	readyIds: [],
	readyTotal: 0,
	sync: null,
	error: null,
});

function reset() {
	party.active = false;
	party.phase = 'disconnected';
	party.roomCode = null;
	party.myId = null;
	party.role = null;
	party.participantCount = 0;
	party.content = null;
	party.readyIds = [];
	party.readyTotal = 0;
	party.sync = null;
	party.error = null;
}

function send(msg: ClientMsg) {
	if (ws?.readyState === WebSocket.OPEN) {
		if (msg.type === 'play' || msg.type === 'pause' || msg.type === 'seek') {
			console.log('[WP:ws-send]', msg.type, 'position' in msg ? (msg as any).position.toFixed(1) : '');
		}
		ws.send(JSON.stringify(msg));
	}
}

function connect(url: string) {
	disconnect();
	party.active = true;
	party.phase = 'connecting';
	party.error = null;

	ws = new WebSocket(url);

	ws.onmessage = (event) => {
		const msg: ServerMsg = JSON.parse(event.data);
		if (msg.type === 'sync') console.log('[WP:ws-recv]', msg.type, { playing: msg.playing, pos: msg.position.toFixed(1) });
		handleMessage(msg);
	};

	ws.onclose = () => {
		if (party.phase !== 'disconnected') {
			party.phase = 'disconnected';
		}
	};

	ws.onerror = () => {
		party.error = 'Connection failed';
		party.phase = 'disconnected';
	};
}

function handleMessage(msg: ServerMsg) {
	switch (msg.type) {
		case 'room_created':
			party.roomCode = msg.code;
			party.phase = 'lobby';
			break;

		case 'welcome':
			party.myId = msg.id;
			party.role = msg.role;
			if (party.phase === 'connecting') {
				party.phase = party.content ? 'watching' : 'picking';
			}
			break;

		case 'participant_joined':
			party.participantCount = msg.count;
			if (party.phase === 'lobby') {
				party.phase = 'picking';
			}
			// Host: sync current page to newly joined guest
			if (party.role === 'host') {
				send({ type: 'navigate', url: window.location.pathname + window.location.search });
			}
			break;

		case 'participant_left':
			party.participantCount = msg.count;
			break;

		case 'room_closed':
			party.phase = 'disconnected';
			party.error = msg.reason;
			break;

		case 'content_set':
			party.content = {
				media_type: msg.media_type,
				tmdb_id: msg.tmdb_id,
				title: msg.title,
				poster_path: msg.poster_path,
				info_hash: msg.info_hash,
				file_idx: msg.file_idx,
				season: msg.season,
				episode: msg.episode,
			};
			party.phase = 'watching';
			break;

		case 'ready_state':
			party.readyIds = msg.ready;
			party.readyTotal = msg.total;
			if (msg.ready.length === msg.total && msg.total >= 2) {
				party.phase = 'watching';
			}
			break;

		case 'navigate_sync':
			// Guest: follow host navigation
			if (party.role === 'guest') {
				goto(msg.url, { replaceState: true });
			}
			break;

		case 'sync':
			party.sync = {
				playing: msg.playing,
				position: msg.position,
				serverTime: msg.server_time,
			};
			break;

		case 'error':
			party.error = msg.message;
			break;

		case 'pong':
			break;
	}
}

// ── Public API ──

export function createRoom() {
	const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
	connect(`${protocol}//${location.host}/cinema/api/watch-party/create`);
}

export function joinRoom(code: string) {
	const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
	connect(`${protocol}//${location.host}/cinema/api/watch-party/join/${code.toUpperCase()}`);
}

export function disconnect() {
	ws?.close();
	ws = null;
	reset();
}

export function navigate(url: string) {
	if (party.active && party.role === 'host') {
		send({ type: 'navigate', url });
	}
}

export function setContent(content: ContentPick) {
	send({
		type: 'set_content',
		media_type: content.media_type,
		tmdb_id: content.tmdb_id,
		title: content.title,
		poster_path: content.poster_path,
		info_hash: content.info_hash,
		file_idx: content.file_idx,
		season: content.season,
		episode: content.episode,
	});
}

export function sendLoaded() {
	send({ type: 'loaded' });
}

export function setReady() {
	send({ type: 'ready' });
}

export function setUnready() {
	send({ type: 'unready' });
}

export function sendPlay(position: number) {
	send({ type: 'play', position });
}

export function sendPause(position: number) {
	send({ type: 'pause', position });
}

export function sendSeek(position: number) {
	send({ type: 'seek', position });
}
