/** Shared reactive flag — set when a page reload is needed after an update or bcachefs switch. */
let _needed = $state(false);

export const refreshState = {
	get needed() { return _needed; },
	set() { _needed = true; },
	clear() { _needed = false; },
};
