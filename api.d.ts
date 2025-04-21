/*
 * api.d.ts
 * (c) 2025 Teodor Potancok
 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/**
 * List of all the supported commands with their argument and return types. This is the main API documentation and source of truth for everything else.
 */
type RpcCommands = {
	/**
	 * Adds a shortcut (non-Steam game) to your library.
	 */
	AddShortcut: {
		args: {
			/** game title */
			name: string;
			/** path to the game executable */
			exe: string;
			/** launch options for the game */
			launchOptions: string[];
			/** path to the game icon (.jpg, .png, .exe, .bmp) */
			icon: string;
			/** path to starting folder */
			startDir: string;
		};
		returns: {
			/** generated app ID */
			appId: number;
		};
	};
	/**
	 * Removes a shortcut from your library.
	 */
	RemoveShortcut: {
		args: {
			/** app ID to remove */
			appId: number;
		};
		returns: {};
	};
	/**
	 * Opens a dialog to install a single app.
	 */
	InstallApp: {
		args: {
			/** app ID to install */
			appId: number;
		};
		returns: {};
	};
	/**
	 * Opens a dialog to install one or more apps. If you pass more than one ID Steam will show them all in one dialog.
	 */
	InstallApps: {
		args: {
			/** app IDs to install */
			appIds: number[];
		};
		returns: {};
	};
	/**
	 * Opens a dialog to uninstall a single app. If `autoConfirm` is `true` uninstalls without asking.
	 */
	UninstallApp: {
		args: {
			/** app ID to remove */
			appId: number;
			/** uninstall without asking? */
			autoConfirm?: boolean | undefined;
		};
		returns: {};
	};
	/**
	 * Opens a dialog to uninstall one or more apps. If `autoConfirm` is `true` uninstalls without asking.
	 */
	UninstallApps: {
		args: {
			/** app ID to remove */
			appIds: number[];
			/** uninstall without asking? */
			autoConfirm?: boolean | undefined;
		};
		returns: {};
	};
	/**
	 * Starts an installed app or shortcut.
	 */
	RunApp: {
		args: {
			/** app ID to run */
			appId: number;
		};
		returns: {};
	};
	/**
	 * Stops a running app or shortcut.
	 */
	TerminateApp: {
		args: {
			/** app ID to stop */
			appId: number;
		};
		returns: {};
	};
	/**
	 * Returns the IDs of all installed apps including shortcuts.
	 */
	GetInstalledApps: {
		args: {};
		returns: {
			/** installed app IDs */
			appIds: number[];
		};
	};
	/**
	 * Returns the IDs of all installed Steam games.
	 */
	GetInstalledGames: {
		args: {};
		returns: {
			/** installed game IDs */
			appIds: number[];
		};
	};
	/**
	 * Sets Steam's UI mode. **This will disconnect Conductor for 1-2 seconds** if the mode changes.
	 */
	SetUIMode: {
		args: {
			/** new UI mode */
			mode: UIMode;
		};
		returns: {};
	};
	/**
	 * Returns Steam's UI mode.
	 */
	GetUIMode: {
		args: {};
		returns: {
			/** current UI mode */
			mode: UIMode;
		};
	};
	GetAppInfo: {
		args: {
			appId: number;
		};
		returns: {
			type: AppType;
			installed: boolean;
			displayName: string;
			storeTags: number[];
		};
	};
};

/**
 * Union of all supported commands
 */
export type Command = keyof RpcCommands;

/**
 * Arguments for a command
 */
export type Args<T extends Command> = RpcCommands[T]["args"];
/**
 * Return values for a command
 */
export type Returns<T extends Command> = RpcCommands[T]["returns"];

/**
 * Request sent to Conductor for command `C`
 */
export type RpcRequest<C extends Command> = {
	command: C;
	args: Args<C>;
};

/**
 * Response returned to the client for command `C`
 */
export type RpcResponse<C extends Command> =
	| { success: false; error: string }
	| ({ success: true } & Returns<C>);

/**
 * Internal handler for command `C`
 */
export type RpcHandler<C extends Command> = (
	request: RpcRequest<C>,
) => Promise<RpcResponse<C>>;

/**
 * Object that maps command names to their handlers
 */
export type RpcHandlers = {
	[C in Command]: RpcHandler<C>;
};
