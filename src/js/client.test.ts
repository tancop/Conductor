import type { Command, Returns, RpcRequest, RpcResponse } from "./api";
import { AppType } from "./api";

/**
 * Type safe RPC client for Conductor. Handles the tricky parts for you so you can just do this:
 *
 * ```
 *    let res = await client.call({
 *        command: "GetApps",
 *        args: {
 *            typeFilter: [AppType.Game, AppType.Shortcut],
 *            installedOnly: true,
 *        },
 *    });
 *
 * client.close();
 * ```
 */
class Client {
    private readonly secret?: string = undefined;
    private ws: WebSocket;
    private readonly ready: Promise<void>;
    private readonly maxConnections: number;
    private counter = 0;

    /**
     * Creates a new client. The default value for `maxConnections` is 500, this should be enough
     * unless you're fetching data for all the apps in your massive library at the same time.
     *
     * @param url the Conductor server's URL
     * @param secret optional secret to send with every request
     * @param maxConnections maximum number of calls waiting at the same time
     */
    constructor(url: string, secret?: string, maxConnections = 500) {
        this.ws = new WebSocket(url);
        this.ready = new Promise((resolve) => {
            this.ws.addEventListener("open", () => resolve());
        });

        this.maxConnections = maxConnections;

        if (secret) {
            this.secret = secret;
        }
    }

    /**
     * Calls a RPC command using this client.
     *
     * @param req request object with command and args
     * @returns response with the command's return value or error
     */
    async call<T extends Command>(req: RpcRequest<T>): Promise<RpcResponse<T>> {
        await this.ready;

        console.log("ready");

        let messageId = this.counter;

        this.counter++;

        if (this.counter === this.maxConnections) {
            this.counter = 0;
        }

        if (this.secret) {
            this.ws.send(
                JSON.stringify({
                    secret: this.secret,
                    messageId,
                    ...req,
                }),
            );
        } else {
            this.ws.send(JSON.stringify({ messageId, ...req }));
        }

        console.log("sent message");

        return new Promise((resolve) => {
            let listener = (event: MessageEvent) => {
                console.log("received message");
                let data = JSON.parse(event.data);

                if (
                    Object.hasOwn(data, "messageId") &&
                    data.messageId === messageId
                ) {
                    this.ws.removeEventListener("message", listener);
                    resolve(data);
                }
            };

            this.ws.addEventListener("message", listener);
        });
    }

    /**
     * Closes the client. You should always do this after you're done with all your requests.
     */
    close() {
        this.ws.close();
    }
}

let client = new Client("ws://localhost:7355", "qyHY9btYEm+6zby4KdGfDQ==");

console.log("created client");

let res = await client.call({
    command: "GetApps",
    args: {
        typeFilter: [AppType.Game, AppType.Shortcut],
    },
});

console.log("called GetApps");

if (res.success) {
    const chunkSize = 100;

    let chunkCount = Math.ceil(res.appIds.length / chunkSize);

    let apps: (Returns<"GetAppInfo"> & { id: number })[] = [];

    performance.mark("fetch");
    for (let i = 0; i < chunkCount; i++) {
        await Promise.all(
            res.appIds.slice(i * chunkSize, (i + 1) * chunkSize).map((appId) =>
                client
                    .call({
                        command: "GetAppInfo",
                        args: {
                            appId,
                        },
                    })
                    .then((app) => {
                        if (app.success) {
                            apps.push({ id: appId, ...app });
                        } else {
                            console.error("GetAppInfo failed");
                        }
                    }),
            ),
        );
    }
    let time = performance.measure("fetch");

    for (const app of apps) {
        console.log(`${app.displayName} [${app.id}]`);
    }

    console.log(`\ntime to fetch library: ${time.duration} ms`);
} else {
    console.error("GetApps failed");
}

client.close();
