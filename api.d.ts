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

type RpcRequest = { secret: string, messageId: number } &
    ({
        command: 'AddShortcut',
        args: {
            name: string,
            exe: string,
            launchOptions: string[],
            icon: string,
            startDir: string,
        },
    } | {
        command: 'RemoveShortcut',
        args: {
            appId: number,
        },
    } | {
        command: 'InstallApp',
        args: {
            appId: number,
        },
    } | {
        command: 'InstallApps',
        args: {
            appIds: number[],
        },
    } | {
        command: 'UninstallApp',
        args: {
            appId: number,
            autoConfirm?: boolean | undefined,
        },
    } | {
        command: 'UninstallApps',
        args: {
            appIds: number[],
            autoConfirm?: boolean | undefined,
        },
    } | {
        command: 'RunApp',
        args: {
            appId: number,
        },
    } | {
        command: 'TerminateApp',
        args: {
            appId: number,
        },
    } | {
        command: 'GetInstalledApps',
    } | {
        command: 'GetInstalledGames',
    } | {
        command: 'EnterGamepadUI',
    } | {
        command: 'ExitGamepadUI',
    } | {
        command: 'IsGamepadUI',
    } | {
        // make it possible to accept invalid commands but not create them
        command: string,
        args: never
    })