/*
 * api.d.ts
 * (c) 2025 Teodor Potancok
 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/**
 * This file documents all of SteamyRPC's commands with their argument and return types
 */

type RpcCommands = {
    AddShortcut: {
        args: {
            name: string,
            exe: string,
            launchOptions: string[],
            icon: string,
            startDir: string,
        },
        returns: {
            appId: number,
        },
    },
    RemoveShortcut: {
        args: {
            appId: number,
        },
        returns: {},
    },
    InstallApp: {
        args: {
            appId: number,
        },
        returns: {},
    },
    InstallApps: {
        args: {
            appIds: number[],
        },
        returns: {},
    },
    UninstallApp: {
        args: {
            appId: number,
            autoConfirm?: boolean | undefined,
        },
        returns: {},
    },
    UninstallApps: {
        args: {
            appIds: number[],
            autoConfirm?: boolean | undefined,
        },
        returns: {},
    },
    RunApp: {
        args: {
            appId: number,
        },
        returns: {},
    },
    TerminateApp: {
        args: {
            appId: number,
        },
        returns: {},
    },
    GetInstalledApps: {
        args: {},
        returns: {
            appIds: number[],
        },
    },
    GetInstalledGames: {
        args: {},
        returns: {
            appIds: number[],
        },
    },
    EnterGamepadUI: {
        args: {},
        returns: {},
    },
    ExitGamepadUI: {
        args: {},
        returns: {},
    },
    IsGamepadUI: {
        args: {},
        returns: {
            isGamepadUI: boolean,
        },
    },
}

type Command = keyof RpcCommands

type Args<T extends Command> = RpcCommands[T]["args"]
type Returns<T extends Command> = RpcCommands[T]["returns"]

type RpcRequest<C extends Command> = {
    secret: string,
    messageId: number,
    command: C,
    args: Args<C>
}

type RpcResponse<C extends Command> = { success: false, error: string } | { success: true } & Returns<C>

type RpcHandler<C extends Command> = (request: RpcRequest<C>) => Promise<RpcResponse<C>>

export type RpcHandlers = {
    [C in Command]: RpcHandler<C>
}