/*
 *  api.ts
 *  (c) 2025 Teodor Potancok
 *
 *  This Source Code Form is subject to the terms of the Mozilla Public
 *  License, v. 2.0. If a copy of the MPL was not distributed with this
 *  file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import type { UIMode } from "./steam";

/**
 * List of all the supported commands with their argument and return types. This is the main API documentation and source of truth for everything else.
 *
 * Commands with a `fields` argument support output filtering like this:
 *
 * ```typescript
 * {
 *     fields: ['driveName', 'userLabel'],
 * }
 *
 * // returns:
 * [
 *     { driveName: 'C:', userLabel: 'main', },
 * ]
 * ```
 */
type RpcCommands = {
    /**
     * Adds a shortcut (non-Steam game) to your library.
     */
    AddShortcut: {
        args: {
            /** game title (default: exe name without extension) */
            name?: string;
            /** path to the game executable */
            exe: string;
            /** launch options for the game (default: []) */
            launchOptions?: string[];
            /** path to the game icon (.jpg, .png, .exe or .bmp, default: game exe) */
            icon?: string;
            /** path to starting folder (default: the exe's parent folder) */
            startDir?: string;
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
            /** index of the folder to install to, asks the user if not set */
            folderIdx?: number;
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
            /** index of the folder to install to, asks the user if not set */
            folderIdx?: number;
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
            autoConfirm?: boolean;
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
            autoConfirm?: boolean;
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
     * Returns the IDs for all the apps in your Steam library that match the filters.
     */
    GetApps: {
        args: {
            /** app types to include in the results (default: all types) */
            typeFilter?: AppType[];
            /** only return apps installed on this device (default: false) */
            installedOnly: boolean;
        };
        returns: {
            /** library app IDs */
            appIds: number[];
        };
    };
    /**
     * Sets Steam's UI mode. **This will disconnect Conductor for 1-2 seconds on Windows** if the mode changes.
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
    /**
     * Returns info about an app in your library.
     */
    GetAppInfo: {
        args: {
            /** the app ID you're looking for */
            appId: number;
        };
        returns: {
            /** app ID */
            id: number;
            /** is installed on this device? */
            installed: boolean;
            /** app display name */
            displayName: string;
            /** the app's store tags */
            storeTags: number[];
            /** launch options passed to the app */
            launchOptions: string;
        } & (
            | {
                  /** the app's type (shortcut) */
                  type: AppType.Shortcut;
                  /** the file this shortcut points to */
                  shortcutExe: string;
                  /** folder used to start the app */
                  startDir: string;
              }
            | {
                  /** the app's type (game, software, tools...) */
                  type: Exclude<AppType, AppType.Shortcut>;
                  /** developer name on Steam */
                  developerName: string;
                  /** index of the folder this app is installed to */
                  installFolder: number;
              }
        );
    };
    /**
     * Returns the localized name of a store tag.
     */
    GetTagName: {
        args: {
            /** the tag's ID */
            tagId: number;
        };
        returns: {
            /** the tag's name */
            name: string;
        };
    };
    /**
     * Returns the localized names for a list of store tags.
     */
    GetTagNames: {
        args: {
            /** the tag IDs */
            tagIds: number[];
        };
        returns: {
            /** names for each ID */
            names: string[];
        };
    };
    /**
     * Returns info about library folders on this device.
     */
    GetInstallFolders: {
        args: {
            /** filter used to pick some fields (default: all fields) */
            fields?: (keyof InstallFolder)[];
        };
        returns: {
            /** filtered data for every folder */
            folders: Partial<InstallFolder[]>;
        };
    };
};

interface InstallFolder {
    index: number;
    isDefault: boolean;
    isRemovable: boolean;
    isMounted: boolean;
    freeSpace: number;
    spaceUsedBySteam: number;
    totalCapacity: number;
    driveName: string;
    folderPath: string;
    userLabel: string;
}

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
 * Request with no secret sent to Steam
 */
export type RpcRequest<C extends Command> = {
    command: C;
    args: Partial<Args<C>>;
};

/**
 * Request sent by a client with an optional secret and message ID
 */
export type ClientRequest<C extends Command> = RpcRequest<C> & {
    secret?: string;
    messageId?: number;
};

/**
 * Response returned to the client for command `C`
 */
export type RpcResponse<C extends Command> =
    | { success: false; error: string; messageId?: number }
    | ({ success: true; messageId?: number } & Returns<C>);

/**
 * Internal handler for command `C`
 */
type RpcHandler<C extends Command> = (
    request: RpcRequest<C>,
) => Promise<RpcResponse<C>>;

/**
 * Object that maps command names to their handlers
 */
export type RpcHandlers = {
    [C in Command]: RpcHandler<C>;
};

/**
 * App types used to control some behavior around opening, closing and configuring apps
 */
export enum AppType {
    /** Games installed from Steam */
    Game = 1,

    /** Non-game software including SFM and VR drivers */
    Software = 2,

    /** Dedicated servers, mod tools, Proton and Steam runtimes. Also includes delisted Half-Life 2 episodes and some Valve software */
    Tool = 4,

    /** Game demos */
    Demo = 8,

    /** Steam features like news and game notes */
    ClientFeature = 256,

    /** Playtests downloaded as a separate app */
    Playtest = 65536,

    /** Non-Steam apps */
    Shortcut = 1073741824,
}
