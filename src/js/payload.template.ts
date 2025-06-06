/*
 *  payload.template.ts
 *  (c) 2025 Teodor Potancok
 *
 *  This Source Code Form is subject to the terms of the Mozilla Public
 *  License, v. 2.0. If a copy of the MPL was not distributed with this
 *  file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import type { RpcHandlers, RpcResponse } from "./api";
import { type AppDetails, AppType } from "./steam";

(() => {
    console.log("ready:", App.GetServicesInitialized());
    if (window.rpc && window.rpc.readyState === WebSocket.OPEN) {
        if ($REPLACE) {
            if (window.rpcReady === false) {
                console.log("Replacing before init is not allowed");
                return;
            }
            console.log("Closing open socket");
            window.rpc.close();
        } else {
            console.log("Ignoring duplicate send");
            return;
        }
    }

    window.rpcReady = false;

    console.log("Opening new socket");

    let ws = new WebSocket("ws://$HOSTNAME");

    window.terminate = () => {
        window.rpc?.send("Terminate");
        window.rpc = undefined;
    };

    function getFields<T, K extends (keyof T)[]>(
        obj: T,
        fields: K | undefined,
    ): Pick<T, K[number]> {
        if (!fields) {
            return obj;
        }

        let filtered: Partial<Pick<T, K[number]>> = {};

        for (const key of fields) {
            filtered[key] = obj[key];
        }

        return filtered as Pick<T, K[number]>;
    }

    async function getAppDetails(appId: number): Promise<AppDetails> {
        return new Promise((resolve, reject) => {
            let appFound = false;
            for (const id of window.appStore.m_mapApps.data_.keys()) {
                if (id === appId) {
                    appFound = true;
                    break;
                }
            }

            if (!appFound) {
                reject(`App with ID ${appId} not found in library`);
                return;
            }

            SteamClient.Apps.RegisterForAppDetails(appId, (e: AppDetails) => {
                resolve(e);
            });
        });
    }

    let handlers: RpcHandlers = {
        AddShortcut: async (msg) => {
            if (!msg.args.exe) {
                return {
                    success: false,
                    error: 'Missing argument "exe"',
                };
            }

            let appId = await SteamClient.Apps.AddShortcut(
                msg.args.name ?? "",
                msg.args.exe,
                (msg.args.launchOptions ?? []).join(" "),
                msg.args.exe,
            );
            if (msg.args.name) {
                SteamClient.Apps.SetShortcutName(appId, msg.args.name);
            }
            if (msg.args.icon) {
                SteamClient.Apps.SetShortcutIcon(appId, msg.args.icon);
            }
            if (msg.args.startDir) {
                SteamClient.Apps.SetShortcutStartDir(appId, msg.args.startDir);
            }

            return {
                success: true,
                appId,
            };
        },
        RemoveShortcut: async (msg) => {
            if (!msg.args.appId) {
                return {
                    success: false,
                    error: 'Missing argument "appId"',
                };
            }

            SteamClient.Apps.RemoveShortcut(msg.args.appId);

            return {
                success: true,
            };
        },
        InstallApp: async (msg) => {
            if (!msg.args.appId) {
                return {
                    success: false,
                    error: 'Missing argument "appId"',
                };
            }

            SteamClient.Installs.OpenInstallWizard([msg.args.appId]);

            if (msg.args.folderIdx) {
                await SteamClient.Installs.SetInstallFolder(
                    msg.args.folderIdx,
                );
                SteamClient.Installs.ContinueInstall();
            }

            return {
                success: true,
            };
        },
        InstallApps: async (msg) => {
            if (!msg.args.appIds) {
                return {
                    success: false,
                    error: 'Missing argument "appId"',
                };
            }

            SteamClient.Installs.OpenInstallWizard(msg.args.appIds);

            if (msg.args.folderIdx) {
                await SteamClient.Installs.SetInstallFolder(
                    msg.args.folderIdx,
                );
                SteamClient.Installs.ContinueInstall();
            }

            return {
                success: true,
            };
        },
        UninstallApp: async (msg) => {
            if (!msg.args.appId) {
                return {
                    success: false,
                    error: 'Missing argument "appId"',
                };
            }

            SteamClient.Installs.OpenUninstallWizard(
                [msg.args.appId],
                msg.args.autoConfirm ?? false,
            );

            return {
                success: true,
            };
        },
        UninstallApps: async (msg) => {
            if (!msg.args.appIds) {
                return {
                    success: false,
                    error: 'Missing argument "appId"',
                };
            }

            SteamClient.Installs.OpenUninstallWizard(
                msg.args.appIds,
                msg.args.autoConfirm ?? false,
            );

            return {
                success: true,
            };
        },
        RunApp: async (msg) => {
            if (!msg.args.appId) {
                return {
                    success: false,
                    error: 'Missing argument "appId"',
                };
            }

            let apps = appStore.m_mapApps.data_;

            let appEntry = apps.get(msg.args.appId);

            if (!appEntry || !appEntry.value_.installed) {
                // app not installed
                return {
                    success: false,
                    error: `App with ID ${msg.args.appId} not installed`,
                };
            }

            let app = appEntry.value_;

            SteamClient.Apps.RunGame(app.gameid, "", -1, 500);

            return {
                success: true,
            };
        },
        TerminateApp: async (msg) => {
            if (!msg.args.appId) {
                return {
                    success: false,
                    error: 'Missing argument "appId"',
                };
            }

            let apps = appStore.m_mapApps.data_;

            let appEntry = apps.get(msg.args.appId);

            if (!appEntry) {
                return {
                    success: false,
                    error: `App with ID ${msg.args.appId} not installed`,
                };
            }

            let app = appEntry.value_;

            SteamClient.Apps.TerminateApp(app.gameid, false);

            return {
                success: true,
            };
        },
        GetApps: async (msg) => {
            if (msg.args?.typeFilter) {
                let filter = new Set(msg.args.typeFilter);

                return {
                    success: true,
                    appIds: appStore.allApps
                        .filter(
                            (app) =>
                                filter.has(app.app_type) &&
                                (!(msg.args.installedOnly ?? false) ||
                                    app.installed),
                        )
                        .map((app) => app.appid),
                };
            }

            if (msg.args?.installedOnly) {
                return {
                    success: true,
                    appIds: appStore.allApps
                        .filter((app) => app.installed)
                        .map((app) => app.appid),
                };
            }

            return {
                success: true,
                appIds: appStore.allApps.map((app) => app.appid),
            };
        },
        SetUIMode: async (msg) => {
            if (!msg.args.mode) {
                return {
                    success: false,
                    error: 'Missing argument "mode"',
                };
            }

            SteamClient.UI.SetUIMode(msg.args.mode);

            return { success: true };
        },
        GetUIMode: async () => {
            return { success: true, mode: await SteamClient.UI.GetUIMode() };
        },
        GetAppInfo: async (msg) => {
            if (!msg.args.appId) {
                return {
                    success: false,
                    error: 'Missing argument "appId"',
                };
            }

            let appEntry = appStore.m_mapApps.data_.get(msg.args.appId);

            if (!appEntry) {
                return {
                    success: false,
                    error: `App with ID ${msg.args.appId} not found in library`,
                };
            }

            let app = appEntry.value_;

            let details = await getAppDetails(msg.args.appId);

            let result: RpcResponse<"GetAppInfo">;

            if (app.app_type === AppType.Shortcut) {
                result = {
                    success: true,
                    id: msg.args.appId,
                    type: app.app_type,
                    installed: app.installed ?? false,
                    displayName: app.display_name,
                    storeTags: app.store_tag,
                    launchOptions: details.strLaunchOptions,
                    // biome-ignore lint/style/noNonNullAssertion: always defined on a shortcut entry
                    shortcutExe: details.strShortcutExe!,
                    // biome-ignore lint/style/noNonNullAssertion: always defined on a shortcut entry
                    startDir: details.strShortcutStartDir!,
                };
            } else {
                result = {
                    success: true,
                    id: msg.args.appId,
                    type: app.app_type,
                    installed: app.installed ?? false,
                    displayName: app.display_name,
                    storeTags: app.store_tag,
                    launchOptions: details.strLaunchOptions,
                    developerName: details.strDeveloperName,
                    installFolder: details.iInstallFolder,
                };
            }

            return result;
        },
        GetTagName: async (msg) => {
            if (!msg.args.tagId) {
                return {
                    success: false,
                    error: 'Missing argument "tagId"',
                };
            }

            return {
                success: true,
                name: appStore.GetLocalizationForStoreTag(msg.args.tagId),
            };
        },
        GetTagNames: async (msg) => {
            return {
                success: true,
                names:
                    msg.args.tagIds?.map((id) =>
                        appStore.GetLocalizationForStoreTag(id),
                    ) ?? [],
            };
        },
        GetInstallFolders: async (msg) => {
            let steamFolders =
                await SteamClient.InstallFolder.GetInstallFolders();

            let folders = steamFolders.map((folder) =>
                getFields(
                    {
                        index: folder.nFolderIndex,
                        isDefault: folder.bIsDefaultFolder,
                        isRemovable: !folder.bIsFixed,
                        isMounted: folder.bIsMounted,
                        freeSpace: folder.nFreeSpace,
                        spaceUsedBySteam: folder.nUsedSize,
                        totalCapacity: folder.nCapacity,
                        driveName: folder.strDriveName,
                        folderPath: folder.strFolderPath,
                        userLabel: folder.strUserLabel,
                    },
                    msg.args.fields,
                ),
            );

            return {
                success: true,
                folders,
            };
        },
    };

    async function handleMessage(msg: {
        command?: string | undefined;
        messageId: number;
    }) {
        if (msg.command) {
            if (!App.GetServicesInitialized()) {
                ws.send(
                    JSON.stringify({
                        messageId: msg.messageId,
                        success: false,
                        error: "Steam is not ready, try again later",
                    }),
                );
                return;
            }

            let handler:
                | ((request: typeof msg) => Promise<object>)
                | undefined =
                // @ts-ignore: `handler` is the requested handler for a valid command or `undefined` for an invalid one
                handlers[msg.command];

            if (handler) {
                try {
                    let response = await handler(msg).catch((reason) => {
                        ws.send(
                            JSON.stringify({
                                messageId: msg.messageId,
                                success: false,
                                error: `Command failed with ${reason}`,
                            }),
                        );
                    });
                    ws.send(
                        JSON.stringify({
                            messageId: msg.messageId,
                            ...response,
                        }),
                    );
                } catch (e) {
                    ws.send(
                        JSON.stringify({
                            messageId: msg.messageId,
                            success: false,
                            error: e,
                        }),
                    );
                }
            } else {
                ws.send(
                    JSON.stringify({
                        messageId: msg.messageId,
                        success: false,
                        error: `Invalid command: ${msg.command}`,
                    }),
                );
            }
        } else {
            ws.send(
                JSON.stringify({
                    success: false,
                    error: "Missing command",
                }),
            );
        }
    }

    ws.addEventListener("message", async (event) => {
        if (event.data === "Ready") {
            window.rpcReady = true;
        }
        let msg: {
            secret?: string | undefined;
            command?: string | undefined;
            messageId: number;
        } = JSON.parse(event.data);
        if (msg.secret && msg.secret === "$SECRET") {
            await handleMessage(msg);

            console.log("Executed RPC command:", msg);
        } else {
            console.error("Refused unauthorized RPC command");
        }
    });

    window.rpcSecret = "$SECRET";

    ws.addEventListener("open", () => {
        ws.send("init:$SECRET");
    });

    ws.addEventListener("close", () => {
        window.rpcSecret = undefined;
        window.rpc?.close();
        window.rpc = undefined;
    });

    window.rpc = ws;
})();
