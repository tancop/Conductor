import type { RpcResponse, Command, RpcRequest } from "./api.d";
import { AppType } from "./api.d";

class Client {
	#secret?: string = undefined;
	#ws: WebSocket;
	#ready: Promise<void>;
	#lastMessageId = 0;
	#maxWaitingMessages: number;

	constructor(url: string, secret?: string, maxWaitingMessages = 100) {
		this.#ws = new WebSocket(url);
		this.#ready = new Promise((resolve) => {
			this.#ws.addEventListener("open", () => resolve());
		});
		this.#maxWaitingMessages = maxWaitingMessages;

		if (secret) {
			this.#secret = secret;
		}
	}

	async call<T extends Command>(req: RpcRequest<T>): Promise<RpcResponse<T>> {
		await this.#ready;
		let messageId = this.#lastMessageId;

		if (this.#secret) {
			this.#ws.send(
				JSON.stringify({
					secret: this.#secret,
					messageId,
					...req,
				}),
			);
		} else {
			this.#ws.send(JSON.stringify({ messageId, ...req }));
		}

		this.#lastMessageId++;

		if (this.#lastMessageId === this.#maxWaitingMessages) {
			this.#lastMessageId = 0;
		}

		return new Promise((resolve) => {
			let listener = (event: MessageEvent) => {
				let data = JSON.parse(event.data);

				if (
					Object.hasOwn(data, "messageId") &&
					data.messageId === messageId
				) {
					this.#ws.removeEventListener("message", listener);
					resolve(JSON.parse(event.data));
				}
			};

			this.#ws.addEventListener("message", listener);
		});
	}

	close() {
		this.#ws.close();
	}
}

let client = new Client("ws://localhost:7355", "qyHY9btYEm+6zby4KdGfDQ==");

let res = await client.call({
	command: "GetApps",
	args: {
		typeFilter: [AppType.Game, AppType.Shortcut],
		installedOnly: true,
	},
});

if (res.success) {
	let apps: RpcResponse<"GetAppInfo">[] = await Promise.all(
		res.appIds.map((appId) =>
			client.call({
				command: "GetAppInfo",
				args: {
					appId,
				},
			}),
		),
	);

	for (const app of apps) {
		if (app.success) {
			console.log(app.displayName);
		} else {
			console.error("GetAppInfo failed");
		}
	}
} else {
	console.error("GetApps failed");
}

client.close();
