#!/usr/bin/env python

import asyncio
import json
import requests
import websockets
import secrets

from websockets.asyncio.server import serve

steam_socket: websockets.ServerConnection | None = None
message_map: dict[int, websockets.ServerConnection] = {}
debugger_url = ""
port = 7355
rpc_secret = ""

last_message_id = 0

reconnecting = False
background_tasks = set()

server: websockets.Server


async def reconnect_to_steam():
    global port
    global rpc_secret
    global debugger_url

    rpc_secret = secrets.token_urlsafe(16)

    payload = make_payload(port, rpc_secret)

    tries = 5

    while tries > 0:
        try:
            if reconnecting:
                await send_payload(debugger_url, payload)
            else:
                print("Connection to Steam restored!")
                return

        except (ConnectionRefusedError, websockets.exceptions.InvalidStatus):
            # try again
            pass
        finally:
            await asyncio.sleep(1)
            tries -= 1

    print("Connection to Steam lost, closing...")
    server.close()


def make_handler():
    async def handler(socket: websockets.ServerConnection):
        global steam_socket
        global last_message_id
        global debugger_url
        global payload
        global reconnecting
        global rpc_secret

        try:
            async for message in socket:
                if message == ("init:" + rpc_secret):
                    if steam_socket and not reconnecting:
                        print("Replay attack blocked!")
                    else:
                        steam_socket = socket
                        reconnecting = False
                        print("SteamyRPC initialized!")
                elif message.startswith("init:"):
                    print("Received bad init message!")
                else:
                    if socket == steam_socket:
                        print("Received message from Steam:", message)

                        msg: dict = json.loads(message)
                        id: int = msg["messageId"]

                        if id in message_map:
                            client = message_map[id]
                            message_map.pop(id, None)
                            msg.pop("messageId", None)
                            try:
                                await client.send(json.dumps(msg))
                            except websockets.exceptions.ConnectionClosed:
                                print("Connection to client lost")
                    else:
                        msg: dict = json.loads(message)

                        id = last_message_id
                        last_message_id += 1

                        message_map[id] = socket

                        await steam_socket.send(
                            json.dumps(
                                {
                                    "messageId": id,
                                    "secret": rpc_secret,
                                    "command": msg["command"],
                                    "args": msg["args"],
                                }
                            )
                        )

                        print("Forwarded command to Steam:", message)
        finally:
            if socket == steam_socket:
                if not reconnecting:
                    print("Connection to Steam lost, resending payload...")

                    reconnecting = True

                    task = asyncio.create_task(reconnect_to_steam())
                    background_tasks.add(task)
                    task.add_done_callback(background_tasks.discard)

    return handler


def make_payload(port: int, rpc_secret: str):
    payload = ""
    with open("payload.template.js", "r") as file:
        for line in file.readlines():
            payload += line.replace(r"{{PORT}}", str(port)).replace(
                r"{{SECRET}}", rpc_secret
            )

        return payload


async def send_payload(debugger_url: str, payload: str):
    async with websockets.connect(debugger_url) as ws:
        msg_id = 1

        command = {
            "id": msg_id,
            "method": "Runtime.evaluate",
            "params": {"expression": payload, "awaitPromise": True},
        }

        await ws.send(json.dumps(command))


async def main():
    global debugger_url
    global payload
    global server
    global rpc_secret
    global port

    rpc_secret = secrets.token_urlsafe(16)
    port = 7355

    print("Starting SteamyRPC...")

    server = await serve(make_handler(), "", port)

    js_context_tab = None

    while True:
        try:
            res = requests.get("http://localhost:8080/json")
            res.raise_for_status()
            tabs: list[dict] = res.json()

            found_tab = False

            for tab in tabs:
                if tab["title"] == "SharedJSContext":
                    js_context_tab = tab
                    found_tab = True
                    break

            if found_tab:
                break
            else:
                # none of the tabs are the right one
                raise requests.exceptions.ConnectionError

        except requests.exceptions.ConnectionError:
            print("Connection to Steam client failed, retrying...")
            await asyncio.sleep(1)

    debugger_url = js_context_tab["webSocketDebuggerUrl"]
    print("Sending payload to:", debugger_url)

    payload = make_payload(port, rpc_secret)

    tries = 5
    while True:
        try:
            await send_payload(debugger_url, payload)
            break
        except (ConnectionRefusedError, websockets.exceptions.InvalidStatus):
            print("Failed to send payload, retrying...")
            tries -= 1
            if tries == 0:
                print(
                    "Failed to send payload. Check if Steam is running with remote debugging enabled."
                )
                return

    await server.serve_forever()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except (asyncio.CancelledError, KeyboardInterrupt):
        print("Goodbye!")
