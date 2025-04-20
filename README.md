# Conductor

## 📘 About

Conductor lets you automate Steam from your browser or over the network. Install and run games, manage your libraries, add non-Steam games without restarting and do what `steamcmd` don't, all without changing Steam's files. No manual install, passwords or complicated commands. Thanks to [Decky Loader](https://github.com/SteamDeckHomebrew/decky-loader) for the injection method.

> [!WARNING]
> Conductor uses TCP port 8080 to communicate with Steam and 7355 to listen for user requests. If you're running something that uses these ports at the same time you need to turn it off or change its settings.

## 💽 Installing

1. Install Python
2. Install the [`websockets`](https://pypi.org/project/websockets/) package
3. Download a release zip and unpack anywhere you want
4. Run Conductor. That's it.

## ⚡ Running

Open `run.ps1` or `run.sh` depending on your system. Conductor will connect to Steam right away if it's running, or wait for it to start. You might get a UAC prompt the first time running it on Windows, this is necessary to enable CEF remote debugging and inject code into the Steam web helper.

## 🔗 Connecting

You can connect to the server at `ws://[your ip]:7355`. Requests should be JSON with the command name and arguments:

```json
{
    "command": "RunApp",
    "args": {
        "appId": 730
    }
}
```

Conductor will respond over the socket with a JSON object. The `success` property lets you know if the command worked or not. In case of failure you can get the reason by looking at `error`. When listing installed games with `GetInstalledGames` the response can look like this:

```json
{
    "success": true,
    "appIds": [730, 1245620, 2767030]
}
```

You can find all the supported commands in [api.d.ts](./api.d.ts). This documentation is always up to date thanks to some TypeScript magic, so you can trust it with your life (at your own risk of course).

## 🛠️ Building

1. Install a package manager like [Bun](https://bun.sh), [pnpm](https://pnpm.io) or [npm](https://nodejs.org)
2. Clone the repo or download source code as a zip
3. Build and run with `bun run dev` or type check and build for production with `bun run build`. Replace `bun` with your package manager if you're using a different one

## 💻 Example Code

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
