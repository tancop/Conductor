# Conductor

## 📘 About

Conductor lets you automate Steam from anywhere. Install and run apps, manage your libraries, add non-Steam games
without restarting and more, locally or over the internet. No manual install, passwords or complicated commands. Thanks
to [Decky Loader](https://github.com/SteamDeckHomebrew/decky-loader) for the injection method.

> [!WARNING]
> Conductor uses TCP port 8080 to connect with Steam. If you're running something that uses it at the same time you need
> to turn it off or change its settings.

## 💽 Installing

1. Download a release zip and unpack anywhere you want
2. Run the app. That's it.

Windows Defender might flag the file as `Behavior:Win32/DefenseEvasion.A!ml` sometimes. This is a false positive AI
detection that comes from using `taskkill` to terminate any other instance running at the same time.

## ⚡ Running

Click on the executable or run it on the command line. You can configure Conductor by editing `settings.toml`:

```toml
[conductor]
hostname = "localhost:7355" # host name clients use to connect
payload_path = "dist/payload.template.js" # path to payload js file, relative to the executable
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

You can find a list with all the supported commands in [api.ts](src/js/api.ts).

## 🔒 Authentication

You can control access to the API by setting some tokens in `settings.toml`:

```toml
[auth]
enabled = true
tokens = ["dQw4w9WgXcQ", "694201337"]
```

If you turn on authentication every client request needs to pass one of the listed tokens in the `secret` field.
Commands with no secret or the wrong value will be rejected.

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
