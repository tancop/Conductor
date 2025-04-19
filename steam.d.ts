/*
 * steam.d.ts
 * (c) 2025 Teodor Potancok
 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/**
 * This file defines all the functions and interfaces used in `payload.template.js` together with some other useful ones.
 * It's not a complete reference for the Steam internal API, and might not ever be. Some properties that are less important
 * or useless for scripting (like MobX auto tracking internals) are not documented on purpose to reduce visual noise.
 */

/**
 * UI mode - Desktop or Gamepad (Big Picture)
 */
declare enum UIMode {
    Gamepad = 4,
    Desktop = 7
}

/**
 * Namespace for global functions that call into Steam's native code
 */
declare namespace SteamClient {
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
        function RunGame(gameId: string, _const1: "", _const2: -1, _const3: 500): void;

        /**
         * Terminates a running app or shortcut. Does nothing if there's no running app with that ID.
         * 
         * @param gameId game ID to terminate (see {@link App.gameid})
         * @param _unknown no visible effect, always called with `false` in Steam code
         */
        function TerminateApp(gameId: string, _unknown: boolean): void;

        /**
         * Adds a shortcut to your Steam library.
         * 
         * @param name the shortcut's internal name used to generate its ID.
         * @param exePath path to the executable, the file name without extension is used as the initial display name
         * @param launchOptions launch options passed to the executable as a string
         * @param commandLine value used to generate the app ID together with {@link name}
         * @returns the shortcut's generated app ID
         */
        function AddShortcut(name: string, exePath: string, launchOptions: string, commandLine: string): Promise<number>;

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
        function SetShortcutStartDir(appId: number, startDirPath: string): void;

        /**
         * Removes a shortcut from your library. Does nothing if there's no shortcut with that ID.
         * 
         * @param appId the shortcut's ID
         */
        function RemoveShortcut(appId: number): void;
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
        function OpenUninstallWizard(appIds: number[], autoConfirm: boolean): void;
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
        function GetUIMode(): UIMode;
    }
}

/**
 * Object representing an app in your library, stored in {@link appStore}
 */
interface App {
    /**
     * Internal ID used to run or terminate the app. This is equal to `appId.toString()` for Steam apps but not shortcuts
     */
    gameid: string;
    /**
     * `true` if the app is installed, `false` or `undefined` if not
     */
    installed?: boolean | undefined;
}

/**
 * MobX store with your library data
 */
declare namespace appStore {
    let m_mapApps: {
        data_: Map<number, { value_: App }>
    };
}