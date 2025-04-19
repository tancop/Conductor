/*
 * payload.template.js
 * (c) 2025 Teodor Potancok
 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// @ts-check

(() => {
    if (window.rpc && window.rpc.readyState == WebSocket.OPEN) {
        if ($REPLACE) {
            console.log("Closing open socket");
            window.rpc.close();
        } else {
            console.log("Ignoring duplicate send");
            return;
        }
    }

    console.log("Opening new socket");

    let ws = new WebSocket("ws://localhost:$PORT");

    ws.addEventListener("message", async (event) => {
        let msg = JSON.parse(event.data);
        if (msg.secret === "$SECRET") {
            switch (msg.command) {
                case "AddShortcut": {
                    /** @type {number} */
                    console.log(msg.args)
                    let appId = await SteamClient.Apps.AddShortcut(msg.args.name, msg.args.exe, msg.args.launchOptions.join(" "), msg.args.exe);
                    SteamClient.Apps.SetShortcutName(appId, msg.args.name);
                    SteamClient.Apps.SetShortcutIcon(appId, msg.args.icon);
                    SteamClient.Apps.SetShortcutStartDir(appId, msg.args.startDir);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                        appId,
                    }));
                    break;
                }
                case "RemoveShortcut": {
                    SteamClient.Apps.RemoveShortcut(msg.args.appId);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                    }));
                    break;
                }
                case "InstallApp": {
                    SteamClient.Installs.OpenInstallWizard(msg.args.appIds);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                    }));
                    break;
                }
                case "UninstallApp": {
                    SteamClient.Installs.OpenUninstallWizard(msg.args.appIds, msg.args.autoConfirm);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                    }));
                    break;
                }
                case "RunApp": {
                    let apps = window.appStore.m_mapApps.data_;

                    let appEntry = apps.get(msg.args.appId);

                    if (!appEntry) {
                        // app not installed
                        ws.send(JSON.stringify({
                            messageId: msg.messageId,
                        }));
                        break;
                    }

                    let app = appEntry.value_;

                    SteamClient.Apps.RunGame(app.gameid, "", -1, 500);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                    }));
                    break;
                }
                case "TerminateApp": {
                    let apps = window.appStore.m_mapApps.data_;

                    let appEntry = apps.get(msg.args.appId);

                    if (!appEntry) {
                        ws.send(JSON.stringify({
                            messageId: msg.messageId,
                        }));
                        break;
                    }

                    let app = appEntry.value_;

                    SteamClient.Apps.TerminateApp(app.gameid, false);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                    }));
                    break;
                }
                case "GetInstalledApps": {
                    let apps = window.appStore.m_mapApps.data_;

                    // collect ids where the associated game is installed
                    let installed = [...apps.entries()].reduce(
                        /** @type {(arr: number[], [id, app]: [number, { value_: App }]) => number[]} */
                        (arr, [id, game]) => {
                            if (game.value_.installed) {
                                return [...arr, id];
                            } else {
                                return arr;
                            }
                        }, []);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                        appIds: installed,
                    }));
                    break;
                }
                case "GetInstalledGames": {
                    let apps = window.appStore.m_mapApps.data_;

                    let installed = [...apps.entries()].reduce(
                        /** @type {(arr: number[], [id, app]: [number, { value_: App }]) => number[]} */
                        (arr, [id, game]) => {
                            if (game.value_.installed && game.value_.app_type == 1) {
                                return [...arr, id];
                            } else {
                                return arr;
                            }
                        }, []);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                        appIds: installed,
                    }));
                    break;
                }
                case "EnterGamepadUI": {
                    SteamClient.UI.SetUIMode(4);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                    }));
                    break;
                }
                case "ExitGamepadUI": {
                    SteamClient.UI.SetUIMode(7);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                    }));
                    break;
                }
                case "IsGamepadUI": {
                    let mode = SteamClient.UI.GetUIMode();

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                        isGamepadUI: (mode === 4),
                    }));
                    break;
                }
                default:
                    console.error("Invalid RPC command:", msg.command);
                    return;
            }
            console.log("Executed RPC command:", msg)
        } else {
            console.error("Refused unauthorized RPC command");
        }
    });

    ws.addEventListener("open", () => {
        ws.send("init:$SECRET");
    });

    window.rpc = ws;
})();