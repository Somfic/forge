/** Return true if the back was handled (stepped back within the page), false to fall through to route navigation. */
let _goBack = $state<(() => boolean) | null>(null);

export function getGoBack() {
	return _goBack;
}

export function setGoBack(fn: (() => boolean) | null) {
	_goBack = fn;
}

let _focusSearch = $state<(() => void) | null>(null);

export function getFocusSearch() {
	return _focusSearch;
}

export function setFocusSearch(fn: (() => void) | null) {
	_focusSearch = fn;
}
