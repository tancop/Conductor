if (window.rpc && window.rpc.readyState == WebSocket.OPEN) {
    window.rpc.send("init:{{SECRET}}");
} else {
    let ws = new WebSocket("ws://localhost:{{PORT}}");
    ws.addEventListener("message", async (event) => {
        let msg = JSON.parse(event.data);
        if (msg.secret === "{{SECRET}}") {
            switch (msg.command) {
                case "AddShortcut":
                    let appId = await SteamClient.Apps.AddShortcut(msg.args.name, msg.args.exe, msg.args.launchOptions.join(" "), msg.args.exe);
                    await SteamClient.Apps.SetShortcutName(appId, msg.args.name);
                    await SteamClient.Apps.SetShortcutIcon(appId, msg.args.icon);
                    await SteamClient.Apps.SetShortcutStartDir(appId, msg.args.startDir);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                        appId,
                    }));
                    break;
                case "RemoveShortcut":
                    await SteamClient.Apps.RemoveShortcut(msg.args.appId);

                    ws.send(JSON.stringify({
                        messageId: msg.messageId,
                    }));
                    break;
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
        ws.send("init:{{SECRET}}");
    });

    window.rpc = ws;
}