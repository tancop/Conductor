/*
 * payload.template.js
 * (c) 2025 Teodor Potancok
 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import type { RpcHandlers } from "./api";

(() => {
	console.log("ready:", App.GetServicesInitialized());
	if (window.rpc && window.rpc.readyState === WebSocket.OPEN) {
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

	let handlers: RpcHandlers = {
		AddShortcut: async (msg) => {
			let appId = await SteamClient.Apps.AddShortcut(
				msg.args.name,
				msg.args.exe,
				msg.args.launchOptions.join(" "),
				msg.args.exe,
			);
			SteamClient.Apps.SetShortcutName(appId, msg.args.name);
			SteamClient.Apps.SetShortcutIcon(appId, msg.args.icon);
			SteamClient.Apps.SetShortcutStartDir(appId, msg.args.startDir);

			return {
				success: true,
				appId,
			};
		},
		RemoveShortcut: async (msg) => {
			SteamClient.Apps.RemoveShortcut(msg.args.appId);

			return {
				success: true,
			};
		},
		InstallApp: async (msg) => {
			SteamClient.Installs.OpenInstallWizard([msg.args.appId]);

			return {
				success: true,
			};
		},
		InstallApps: async (msg) => {
			SteamClient.Installs.OpenInstallWizard(msg.args.appIds);

			return {
				success: true,
			};
		},
		UninstallApp: async (msg) => {
			SteamClient.Installs.OpenUninstallWizard(
				[msg.args.appId],
				msg.args.autoConfirm ?? false,
			);

			return {
				success: true,
			};
		},
		UninstallApps: async (msg) => {
			SteamClient.Installs.OpenUninstallWizard(
				msg.args.appIds,
				msg.args.autoConfirm ?? false,
			);

			return {
				success: true,
			};
		},
		RunApp: async (msg) => {
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
		GetInstalledApps: async (msg) => {
			let apps = appStore.m_mapApps.data_;

			// collect ids where the associated game is installed
			let installed = [...apps.entries()].reduce((arr, [id, game]) => {
				if (game.value_.installed) {
					arr.push(id);
				}
				return arr;
			}, [] as number[]);

			return {
				success: true,
				appIds: installed,
			};
		},
		GetInstalledGames: async (msg) => {
			let apps = appStore.m_mapApps.data_;

			let installed = [...apps.entries()].reduce((arr, [id, game]) => {
				if (game.value_.installed && game.value_.app_type === AppType.Game) {
					arr.push(id);
				}
				return arr;
			}, [] as number[]);

			return {
				success: true,
				appIds: installed,
			};
		},
		EnterGamepadUI: async (msg) => {
			SteamClient.UI.SetUIMode(UIMode.Gamepad);

			return {
				success: true,
			};
		},
		ExitGamepadUI: async (msg) => {
			SteamClient.UI.SetUIMode(UIMode.Desktop);

			return {
				success: true,
			};
		},
		IsGamepadUI: async (msg) => {
			let mode = SteamClient.UI.GetUIMode();

			return {
				success: true,
				isGamepadUI: mode === 4,
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

			let handler: ((request: typeof msg) => Promise<object>) | undefined =
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
		let msg: {
			secret?: string | undefined;
			command?: string | undefined;
			messageId: number;
		} = JSON.parse(event.data);
		if (msg.secret && msg.secret === "$SECRET") {
			handleMessage(msg);

			console.log("Executed RPC command:", msg);
		} else {
			console.error("Refused unauthorized RPC command");
		}
	});

	ws.addEventListener("open", () => {
		ws.send("init:$SECRET");
	});

	window.rpc = ws;
})();
