/*
 * steam.ts
 * (c) 2025 Teodor Potancok
 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/*
 * This file defines all the functions and interfaces used in `payload.template.js` together with some other useful ones.
 * It's not a complete reference for the Steam internal API, and might not ever be. Some properties that are less important
 * or useless for scripting (like MobX auto tracking internals) are not documented on purpose to reduce visual noise.
 */

/**
 * Constants used to switch between desktop, Big Picture and closed mode
 */
export declare enum UIMode {
    /**
     * Closes all windows and disables interactions with the tray icon. Opening Steam from the taskbar or a desktop
     * shortcut crashes SteamWebHelper. Might be used to hide Steam while a game is running
     */
    Closed = 1,

    /** Gamepad UI (Big Picture mode). This is the default on Steam Deck */
    Gamepad = 4,

    /** Desktop mode. This is the default on non-Deck devices */
    Desktop = 7,
}

/**
 * Represents a Steam library folder. There can only be one folder for each mounted drive.
 */
export interface InstallFolder {
    /**
     * Index of this folder in the list
     */
    nFolderIndex: number;
    /**
     * `true` if this is the default folder
     */
    bIsDefaultFolder: boolean;

    /**
     * `true` if the folder on a fixed (non-removable) drive
     */
    bIsFixed: boolean;

    /**
     * `true` if the folder's drive is mounted
     */
    bIsMounted: boolean;

    /**
     * Amount of storage used by Steam, in bytes
     */
    nUsedSize: number;

    /**
     * Total capacity of the drive, in bytes
     */
    nCapacity: number;

    /**
     * Free space on the drive, in bytes
     */
    nFreeSpace: number;

    /**
     * The drive's name (this is a drive letter like "C:" on Windows)
     */
    strDriveName: string;

    /**
     * Path to the library folder like "C:\Program Files (x86)\Steam"
     */
    strFolderPath: string;

    /**
     * User label for the folder, empty string if there's no label set
     */
    strUserLabel: string;
}

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

/**
 * Object representing an app in your library, stored in {@link appStore}
 */
export interface App {
    /**
     * The app's type - see {@link AppType} for details
     */
    app_type: AppType;

    /**
     * App ID. Assigned by Valve for store apps or generated from the parameters of {@link SteamClient.Apps.AddShortcut} for shortcuts
     */
    appid: number;

    /**
     * `true` if the app is installed, `false` or `undefined` if not
     */
    installed?: boolean | undefined;

    /**
     * Human-readable display name
     */
    display_name: string;

    /**
     * Internal name used for sorting apps. For most apps it's the lower case version of `display_name` with special characters removed,
     * publishers can change it to get the right order for games in a series
     */
    sort_as: string;

    /**
     * List of tag IDs for this app. You can get their names with {@link appStore.GetLocalizationForStoreTag}
     */
    store_tag: number[];

    /**
     * Internal ID used to run or terminate the app. This is equal to `appid.toString()` for store apps but different for shortcuts
     */
    gameid: string;
}

/**
 * Object representing a user
 */
export interface User {
    /**
     * The user's SteamID64 as a base 10 number
     */
    strSteamID: string;

    /**
     * The user's private account name
     */
    strAccountName: string;

    /**
     * `true` if the user has a limited account
     */
    bIsLimited: boolean;

    /**
     * `true` if the user is offline
     */
    bIsOfflineMode: boolean;
}

declare global {
    /**
     * Namespace for global functions that call into Steam's native code
     */
    namespace SteamClient {
        /**
         * Functions related to shortcuts (non-Steam games) and running apps
         */
        namespace Apps {
            /**
             * Runs an installed app or shortcut. Does nothing if there's no installed app with that ID.
             *
             * @param gameId game ID to run (see {@link App.gameid})
             * @param _const1 always ""
             * @param _const2 always -1
             * @param _const3 always 500
             */
            function RunGame(
                gameId: string,
                _const1: "",
                _const2: -1,
                _const3: 500,
            ): void;

            /**
             * Terminates a running app or shortcut. Does nothing if there's no running app with that ID.
             *
             * @param gameId game ID to terminate (see {@link App.gameid})
             * @param _unknown no visible effect, always called with `false` in Steam code
             */
            function TerminateApp(gameId: string, _unknown: boolean): void;

            /**
             * Adds a shortcut to your Steam library. This will always create a new shortcut even if one already exists with the same parameters.
             * Generated app IDs are deterministic - adding, removing and re-adding a shortcut will produce the same initial value. If a duplicate shortcut
             * exists, Steam will assign a different ID.
             *
             * @param name the shortcut's internal name used to generate its ID.
             * @param exePath path to the executable, the file name without extension is used as the initial display name
             * @param launchOptions launch options passed to the executable as a string
             * @param commandLine value used to generate the app ID together with {@link name}
             * @returns the shortcut's generated app ID
             */
            function AddShortcut(
                name: string,
                exePath: string,
                launchOptions: string,
                commandLine: string,
            ): Promise<number>;

            /**
             * Sets a shortcut's display name. Does nothing if there's no shortcut with that ID in your library.
             *
             * @param appId the shortcut's ID
             * @param name new display name
             */
            function SetShortcutName(appId: number, name: string): void;

            /**
             * Sets a shortcut's icon. Does nothing if there's no shortcut with that ID in your library.
             *
             * **Warning:** passing an invalid path (file doesn't exist or is the wrong type) will break the shortcut's icon until Steam exits.
             * If that happens, restart Steam or remove the shortcut and re-add it with the same parameters. **TGA files are not supported** even
             * though Steam shows them in the file dialog, using them will break the shortcut.
             *
             * @param appId the shortcut's ID
             * @param iconPath path to new icon (.jpg, .png, .exe, .bmp)
             */
            function SetShortcutIcon(appId: number, iconPath: string): void;

            /**
             * Sets the starting folder (Configure -> Start In) for a shortcut. Does nothing if there's no shortcut with that ID in your library.
             *
             * @param appId the shortcut's ID
             * @param startDirPath path to starting folder
             */
            function SetShortcutStartDir(
                appId: number,
                startDirPath: string,
            ): void;

            /**
             * Removes a shortcut from your library. Does nothing if there's no shortcut with that ID.
             *
             * @param appId the shortcut's ID
             */
            function RemoveShortcut(appId: number): void;
        }

        /**
         * Functions for managing install folders
         */
        namespace InstallFolder {
            /**
             * Returns info about all the install folders on this device.
             */
            function GetInstallFolders(): Promise<InstallFolder[]>;

            /**
             * Opens the `steamapps` for a library folder in your default file manager.
             *
             * @param folderIdx index of the folder to open
             */
            function BrowseFilesInFolder(folderIdx: number): void;

            /**
             * Moves an installed app to a different library folder. This doesn't move the app right away,
             * just add a move operation to the update queue.
             *
             * @param appId the moved app's ID
             * @param folderIdx the target folder's index
             */
            function MoveInstallFolderForApp(
                appId: number,
                folderIdx: number,
            ): Promise<number>;
        }

        /**
         * Functions for installing and uninstalling apps
         */
        namespace Installs {
            /**
             * Opens a wizard to install the provided apps. Invalid IDs or already installed apps will be ignored.
             *
             * @param appIds app IDs to install
             */
            function OpenInstallWizard(appIds: number[]): void;

            /**
             * Uninstalls the provided apps if {@link autoConfirm} is `true`, opens the uninstall wizard if it's `false`.
             * Invalid IDs or uninstalled apps will be ignored.
             *
             * @param appIds app IDs to remove
             * @param autoConfirm uninstall without asking the user?
             */
            function OpenUninstallWizard(
                appIds: number[],
                autoConfirm: boolean,
            ): void;
        }

        /**
         * Functions for controlling the UI
         */
        namespace UI {
            /**
             * Changes Steam's current UI mode (desktop or gamepad/big picture). **This restarts the Steam client** and resets SteamWebHelper,
             * so you need to reinject any running payloads.
             *
             * @param mode new mode
             */
            function SetUIMode(mode: UIMode): void;

            /**
             * Returns Steam's current UI mode.
             */
            function GetUIMode(): Promise<UIMode>;
        }
    }

    /**
     * MobX store with your library data
     */
    namespace appStore {
        /**
         * Map from app ID to {@link App} object
         */
        export let m_mapApps: {
            data_: Map<number, { value_: App }>;
        };

        /**
         * Array with all the apps in your library, including shortcuts
         */
        export let allApps: App[];

        /**
         * Gets the localized human-readable name for a store tag like 'Immersive Sim' or 'LGBTQ+'.
         *
         * @param tagId tag ID
         */
        export function GetLocalizationForStoreTag(tagId: number): string;
    }

    /**
     * Stores info about your account and Steam's loading state
     */
    namespace App {
        /**
         * Returns `true` if Steam services are initialized.
         */
        export function GetServicesInitialized(): boolean;

        /**
         * Returns the current user.
         */
        export function GetCurrentUser(): User;
    }
}
