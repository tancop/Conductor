# Conductor

## 📘 About

Conductor lets you automate Steam from your browser or over the network. Install and run games, manage your libraries,
add non-Steam games without restarting and do what `steamcmd` don't, all without changing Steam's files. No manual
install, passwords or complicated commands. Thanks to [Decky Loader](https://github.com/SteamDeckHomebrew/decky-loader)
for the injection method.

> [!WARNING]
> Conductor uses TCP port 8080 to connect with Steam and another one (7355 by default) to communicate
> with clients. If you're running something that uses these ports at the same time you need to turn it
> off or change its settings.

## 💽 Installing

1. Download a release zip and unpack anywhere you want
2. Run Conductor. That's it.

## ⚡ Running

Click on the executable or run it on the command line. There are some useful command line options:

```
-p, --port <PORT>        Port used for opening connections on localhost. The Steam payload will always connect to `ws://localhost:[port]` [default: 7355]
-s, --secret <SECRET>    Secret for client authentication. If this option is set all requests need to have a `secret` field with the provided value
-a, --address <ADDRESS>  Hostname used for client connections. Defaults to `localhost:[port]`
```

## 🔗 Connecting

With default settings you can connect to the server over WebSockets at `ws://localhost:7355`. Requests should be JSON
with the command name and arguments:

```json
{
  "command": "RunApp",
  "args": {
    "appId": 730
  }
}
```

Conductor will respond over the socket with a JSON object. The `success` property lets you know if the command worked or
not. If it failed you can get the reason by looking at `error`. For example, when listing installed games with
`GetInstalledApps` the response can look like this:

```json
{
  "success": true,
  "appIds": [
    730,
    1245620,
    2767030
  ]
}
```

You can find all the supported commands in [api.ts](src/js/api.ts).

## 🔒 Authentication

You can control access to the API by passing a client secret as a command line parameter:

```
conductor -s "dQw4w9WgXcQ"
```

If you set a secret every client request needs to pass it in the `secret` field. Commands with no secret or the wrong
value will be rejected.

```json
{
  "command": "RunApp",
  "args": {
    "appId": 730
  },
  "secret": "dQw4w9WgXcQ"
}
```

## 🛠️ Building

1. Install [Rust](https://www.rust-lang.org/tools/install) and a JS package manager
   like [Bun](https://bun.sh), [pnpm](https://pnpm.io) or [npm](https://nodejs.org)
2. Clone the repo or download source code as a zip
3. Build and run with `bun run dev` or type check and build for production with `bun run build`. Replace `bun` with your
   package manager if you're using a different one.

## 💻 Example Code

If Counter-Strike 2 is installed, this opens it and prints `{"success": true}` to the
console. If it's not, prints `{"success": false, "error": "App with ID 730 not installed"}`

```javascript
let ws = new WebSocket("ws://localhost:7355");

ws.addEventListener("message", (event) => console.log(event.data));

ws.addEventListener("open", () => {
    ws.send(
        JSON.stringify({
            command: "RunApp",
            args: {
                appId: 730,
            },
        }),
    );
});
```
