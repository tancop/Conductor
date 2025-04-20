# SteamyRPC

## About

This is a powerful script that lets you control your Steam client with a WebSocket interface. No more messing with steamcmd or VDFs. Inspired by [Decky Loader](https://github.com/SteamDeckHomebrew/decky-loader).

## Important

SteamyRPC uses TCP port `8080` to communicate with Steam and `7355` to listen for user requests. If you're running a web server or other app that uses these ports at the same time you need to turn them off or change their settings.

## Building

1. Install a package manager like [Bun](https://bun.sh), [pnpm](https://pnpm.io) or [npm](https://nodejs.org)
2. Clone the repo or download source code as a zip
3. Build and run with `bun run dev` or type check and build for production with `bun run build`. Replace `bun` with your package manager if you're using a different one

## Installing

1. Install the [`websockets`](https://pypi.org/project/websockets/) Python package with uv or pip
2. Download a release zip and unpack anywhere you want
3. Use `run.ps1` or `run.sh` depending on your system to start SteamyRPC. You might get a UAC prompt the first time running it on Windows, this is necessary to enable CEF remote debugging and let SteamyRPC connect to Steam.

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

SteamyRPC will respond over the socket with a JSON object. The `success` property lets you know if the command worked or not. In case of failure you can get the reason by looking at `error`. When listing installed games with `GetInstalledGames` the response can look like this:

```json
{
    "success": true,
    "appIds": [730, 1245620, 2767030]
}
```

You can find all the supported commands in [api.d.ts](./api.d.ts). This documentation is always up to date thanks to some TypeScript magic, so you can trust it with your life (at your own risk of course).

## Example Code

If Counter-Strike 2 is installed, this opens it and prints `{"success": true}` to the console. If it's not, prints `{"success": false, "error": "App with ID 730 not installed"}`

```javascript
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
