# SteamyRPC

## About

SteamyRPC is a powerful script that lets you control your Steam client with a WebSocket interface. No more messing with steamcmd or VDFs. Inspired by [Decky Loader](https://github.com/SteamDeckHomebrew/decky-loader).

## Installing

1. Install the [`websockets`](https://pypi.org/project/websockets/) Python package with uv or pip
2. Download a release zip and unpack anywhere you want
3. Create an empty file named `.cef-enable-remote-debugging` in `C:\Program Files (x86)\Steam` on Windows or `~/.steam/steam` on Linux. This turns on remote debugging on the Steam web helper to let SteamyRPC connect.
4. Open Steam
5. Run `server.py`

Repeat the last two steps any time you want to use SteamyRPC again.

## Connecting

You can connect to the server at `ws://[your ip]:7355`. Requests should be JSON with the command name and arguments:

```json
{
    "command": "RunApp",
    "args": {
        "appId": 730
    }
}
```

SteamyRPC will respond over the socket with a JSON object containing the command's return values. Commands that return nothing send an empty object. When creating a shortcut with `AddShortcut` the response can look like this:

```json
{
    "appId": 2314605418
}
```

## Example Code

This opens Counter-Strike 2 if it's installed and prints `{}` to console.

```typescript
let ws = new WebSocket('ws://localhost:7355');

ws.addEventListener('message', (event) => console.log(event.data));

ws.addEventListener('open', () => {
    ws.send(JSON.stringify({
        command: 'RunApp',
        args: {
            appId: 730
        }
    }));
});
```

## Commands

### AddShortcut

Adds a shortcut (non-Steam game) to your library.

**Arguments:**

```typescript
{
    name: string, // game title
    exe: string, // path to the game executable
    launchOptions: string[], // launch options for the game
    icon: string, // path to the game icon (.jpg, .png, .exe, .ico)
}
```

**Returns**:

```typescript
{
    appId: number, // the new shortcut's app ID
}
```

### RemoveShortcut

Removes a shortcut from your library.

**Arguments:**

```typescript
{
    appId: number, // the removed shortcut's app ID
}
```

### InstallApp

Opens a dialog to install one or more apps. If you pass more than one ID Steam will show them in one dialog.

**Arguments:**

```typescript
{
    appIds: number[], // app IDs to be installed
}
```

### UninstallApp

Uninstalls one or more apps if `autoConfirm` is `true`, opens a uninstall dialog for them if it's `false`.

**Arguments:**

```typescript
{
    appIds: number[], // app IDs to be uninstalled
    autoConfirm: boolean, // uninstall without asking the user?
}
```

### RunApp

Starts an installed app. Does nothing if it's not installed.

**Arguments:**

```typescript
{
    appId: number, // app ID to run
}
```

### TerminateApp

Stops an installed app. Does nothing if it's not running.

**Arguments:**

```typescript
{
    appId: number, // app ID to terminate
}
```

### GetInstalledApps

Returns an array with IDs for every installed app, including shortcuts and non-games.

**Returns:**

```typescript
{
    appIds: number[], // ID for every installed app
}
```

### GetInstalledGames

Returns an array with IDs for every app installed from Steam.

**Returns:**

```typescript
{
    appIds: number[], // ID for every installed app
}
```

### EnterGamepadUI

Switches Steam to Big Picture mode (gamepad UI). Does nothing if already in Big Picture. **This will disconnect SteamyRPC for 1-2 seconds.**

### ExitGamepadUI

Switches Steam to normal mode (desktop UI). Does nothing if already in normal mode. **This will disconnect SteamyRPC for 1-2 seconds.**

### IsGamepadUI

Checks if Steam is in Big Picture mode.

**Returns:**

```typescript
{
    isGamepadUI: boolean, // `true` if Steam is in BPM, `false` if not
}
```
