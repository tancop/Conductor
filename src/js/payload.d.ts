interface Window {
	/**
	 * Injected WebSocket instance used to communicate with the Python server
	 */
	rpc: WebSocket | undefined;
}

/**
 * Decides if this payload should replace any previous one or leave it alone. This property gets replaced
 * by a `true` or `false` literal during template substitution.
 */

// biome-ignore lint/style/noVar: global variables are not block scoped
declare var $REPLACE: boolean;
