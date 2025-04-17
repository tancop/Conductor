#!/usr/bin/env python

import asyncio
import json
import requests
import websockets
import secrets
import logging
import sys

from websockets.asyncio.server import serve

MAX_MESSAGE_ID = 100
MAX_PAYLOAD_TRIES = 5
MAX_RECONNECT_TRIES = 5

steam_socket: websockets.ServerConnection | None = None
message_map: dict[int, websockets.ServerConnection] = {}
debugger_url = ""
port = 7355
rpc_secret = ""

last_message_id = 0

closing = False
reconnecting = False
background_tasks = set()

server: websockets.Server


class DowngradeInfoFilter(logging.Filter):
    def filter(self, record):
        if record.levelno == logging.INFO:
            record.levelno = logging.DEBUG


logging.basicConfig(
    stream=sys.stdout,
    level=logging.INFO,
    format="[%(asctime)s] %(levelname)s: %(message)s",
    datefmt="%H:%M:%S",
)

for lib in ["requests", "websockets", "websockets.server"]:
    logger = logging.getLogger(lib)
    logger.addFilter(DowngradeInfoFilter())


logger = logging.getLogger(__name__)


async def reconnect_to_steam():
    global port
    global rpc_secret
    global debugger_url
    global closing

    rpc_secret = secrets.token_urlsafe(16)

    payload = make_payload(port, rpc_secret, False)

    tries = MAX_RECONNECT_TRIES

    while tries > 0:
        try:
            if reconnecting:
                logger.info("Connection to Steam lost, resending payload...")
                await send_payload(debugger_url, payload)
            else:
                return

        except (ConnectionRefusedError, websockets.exceptions.InvalidStatus):
            # try again
            await asyncio.sleep(1)
            tries -= 1

    logger.info("Connection to Steam lost, closing...")

    closing = True

    server.close()


def make_handler():
    async def handler(socket: websockets.ServerConnection):
        global steam_socket
        global last_message_id
        global reconnecting
        global rpc_secret
        global closing

        try:
            async for message in socket:
                if message == ("init:" + rpc_secret):
                    if steam_socket and not reconnecting:
                        logger.warning("Replay attack blocked!")
                    else:
                        steam_socket = socket
                        if reconnecting:
                            reconnecting = False
                            logger.info("Reconnected to Steam!")
                        else:
                            logger.info("SteamyRPC initialized!")

                elif message.startswith("init:"):
                    logger.error("Received bad init message:", message)
                else:
                    if socket == steam_socket:
                        msg: dict = json.loads(message)

                        if "messageId" not in msg:
                            logger.critical("Received message without ID from Steam!")
                            continue

                        id: int = msg["messageId"]

                        if id in message_map:
                            client = message_map[id]
                            message_map.pop(id, None)
                            msg.pop("messageId", None)
                            try:
                                await client.send(json.dumps(msg))
                                logger.debug("Sent response to client:", msg)
                            except websockets.exceptions.ConnectionClosed:
                                logger.warning("Connection to client lost")
                    else:
                        msg: dict
                        try:
                            msg = json.loads(message)
                        except json.JSONDecodeError:
                            logger.error("Received invalid JSON from client:", message)
                            continue

                        if "command" not in msg:
                            logger.error("No command found in message")
                            continue

                        id = last_message_id
                        last_message_id += 1

                        if last_message_id == MAX_MESSAGE_ID:
                            last_message_id = 0

                        message_map[id] = socket

                        if "args" in msg:
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
                        else:
                            await steam_socket.send(
                                json.dumps(
                                    {
                                        "messageId": id,
                                        "secret": rpc_secret,
                                        "command": msg["command"],
                                    }
                                )
                            )

                        logger.debug("Sent command to Steam:", message)
        except asyncio.CancelledError:
            closing = True
        finally:
            if socket == steam_socket:
                if not reconnecting and not closing:
                    reconnecting = True

                    task = asyncio.create_task(reconnect_to_steam())
                    background_tasks.add(task)
                    task.add_done_callback(background_tasks.discard)

    return handler


def make_payload(port: int, rpc_secret: str, replace: bool):
    payload = ""
    try:
        with open("payload.template.js", "r") as file:
            for line in file.readlines():
                payload += (
                    line.replace("$PORT", str(port))
                    .replace("$SECRET", rpc_secret)
                    .replace("$REPLACE", "true" if replace else "false")
                )

            return payload
    except FileNotFoundError:
        logger.critical("Payload file not found!")
        server.close()


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
    global server
    global rpc_secret
    global closing

    rpc_secret = secrets.token_urlsafe(16)

    logger.info("Starting SteamyRPC...")

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
            logger.info("Connection to Steam client failed, retrying...")
            await asyncio.sleep(1)

    if "webSocketDebuggerUrl" not in js_context_tab:
        logger.critical("ERROR: SharedJSContext has no debugger URL!")
        return

    debugger_url = js_context_tab["webSocketDebuggerUrl"]
    logger.debug("Sending payload to:", debugger_url)

    payload = make_payload(port, rpc_secret, True)

    tries = MAX_PAYLOAD_TRIES
    while True:
        try:
            await send_payload(debugger_url, payload)
            break
        except (ConnectionRefusedError, websockets.exceptions.InvalidStatus):
            logger.warning("Failed to send payload, retrying...")
            tries -= 1
            if tries == 0:
                logger.error(
                    "Failed to send payload. Check if Steam is running with remote debugging enabled."
                )
                return

    await server.serve_forever()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except (asyncio.CancelledError, KeyboardInterrupt):
        closing = True
        logger.info("Goodbye!")
