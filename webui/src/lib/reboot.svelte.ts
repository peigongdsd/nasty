/** Shared reactive flag — set when a system restart is needed after a kernel/driver update.
 *  Source of truth is the engine (system.reboot_required); this is just the cached UI state. */
let _needed = $state(false);

export const rebootState = {
	get needed() { return _needed; },
	set() { _needed = true; },
	clear() { _needed = false; },
};
