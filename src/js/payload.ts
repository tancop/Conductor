/*
 *  payload.ts
 *  (c) 2025 Teodor Potancok
 *
 *  This Source Code Form is subject to the terms of the Mozilla Public
 *  License, v. 2.0. If a copy of the MPL was not distributed with this
 *  file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

interface Window {
    /**
     * Injected WebSocket instance used to communicate with the Python server
     */
    rpc: WebSocket | undefined;
    terminate: (() => void) | undefined;
    rpcSecret: string | undefined;
    rpcReady: boolean | undefined;
}

/**
 * Decides if this payload should replace any previous one or leave it alone. This property gets replaced
 * by a `true` or `false` literal during template substitution.
 */

// biome-ignore lint/style/noVar: global variables are not block scoped
declare var $REPLACE: boolean;
