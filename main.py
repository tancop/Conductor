#!/usr/bin/env python

# server.py
# (c) 2025 Teodor Potancok

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import asyncio
import json
import os
from pathlib import Path
import signal
import tempfile
import requests
import websockets
import secrets
import logging
import sys

from websockets.asyncio.server import serve
from requests.adapters import HTTPAdapter

MAX_MESSAGE_ID = 100
MAX_PAYLOAD_TRIES = 5
MAX_RECONNECT_TRIES = 5


class Context:
    steam_socket: websockets.ServerConnection | None = None

    message_map: dict[int, websockets.ServerConnection] = {}
    id_map: dict[int, int] = {}

    debugger_url = ""
    port = 7355
    rpc_secret = ""

    last_message_id = 0

    closing = False
    reconnecting = False
    background_tasks = set()

    server: websockets.Server

    logger: logging.Logger

    api_secrets: list[str] = []


class DowngradeInfoFilter(logging.Filter):
    def filter(self, record: logging.LogRecord):
        if record.levelno == logging.INFO:
            record.levelno = logging.DEBUG
            record.levelname = logging.getLevelName(logging.DEBUG)
        return True


class ColorFormatter(logging.Formatter):
    def format(self, record: logging.LogRecord):
        if record.levelno == logging.INFO:
            msg = f"[{self.formatTime(record, '%H:%M:%S')}] {record.getMessage()}"
        else:
            msg = f"[{self.formatTime(record, '%H:%M:%S')}] {record.levelname}: {record.getMessage()}"

        if record.levelno == logging.WARNING:
            return "\x1b[33;20m" + msg + "\x1b[0m"
        elif record.levelno == logging.ERROR:
            return "\x1b[31;20m" + msg + "\x1b[0m"
        elif record.levelno == logging.CRITICAL:
            return "\x1b[31;1m" + msg + "\x1b[0m"
        else:
            return msg


async def reconnect_to_steam(ctx: Context):
    rpc_secret = secrets.token_urlsafe(16)

    payload = make_payload(ctx.port, rpc_secret, False)

    tries = MAX_RECONNECT_TRIES

    while tries > 0:
        try:
            if ctx.reconnecting:
                ctx.logger.info("Connection to Steam lost, resending payload...")
                await send_payload(ctx.debugger_url, payload)
            else:
                return

        except (ConnectionRefusedError, websockets.exceptions.InvalidStatus):
            # try again
            tries -= 1
            await asyncio.sleep(0.5)

    ctx.logger.info("Connection to Steam lost, closing...")

    ctx.closing = True

    ctx.server.close()


def make_handler(ctx: Context):
    async def handler(socket: websockets.ServerConnection):
        try:
            async for message in socket:
                if message == ("init:" + ctx.rpc_secret):
                    if ctx.steam_socket and not ctx.reconnecting:
                        ctx.logger.warning("Replay attack blocked!")
                    else:
                        ctx.steam_socket = socket
                        if ctx.reconnecting:
                            reconnecting = False
                            ctx.logger.info("Reconnected to Steam!")
                        else:
                            ctx.logger.info("Conductor initialized!")

                elif str(message).startswith("init:"):
                    ctx.logger.error("Received bad init message:", message)
                else:
                    if socket == ctx.steam_socket:
                        msg: dict = json.loads(message)

                        if "messageId" not in msg:
                            ctx.logger.critical(
                                "Received message without ID from Steam!"
                            )
                            continue

                        id: int = msg["messageId"]

                        if id in ctx.message_map:
                            client = ctx.message_map[id]
                            ctx.message_map.pop(id, None)

                            if id in ctx.id_map:
                                msg["messageId"] = ctx.id_map[id]
                                ctx.id_map.pop(id)
                            else:
                                msg.pop("messageId", None)

                            try:
                                await client.send(json.dumps(msg))
                                ctx.logger.debug("Sent response to client: %s" % msg)
                            except websockets.exceptions.ConnectionClosed:
                                ctx.logger.warning("Connection to client lost")
                    else:
                        msg: dict
                        try:
                            msg = json.loads(message)
                        except json.JSONDecodeError:
                            ctx.logger.warning(
                                "Received invalid JSON from client: %s" % message
                            )
                            await socket.send(
                                json.dumps(
                                    {
                                        "success": False,
                                        "error": "Message is not valid JSON",
                                    }
                                )
                            )
                            continue

                        if "command" not in msg:
                            ctx.logger.error("No command found in message")
                            await socket.send(
                                json.dumps(
                                    {
                                        "success": False,
                                        "error": "No command found",
                                    }
                                )
                            )
                            continue

                        if len(ctx.api_secrets) > 0:
                            # authentication enabled
                            if "secret" not in msg:
                                ctx.logger.warning(
                                    "Received client message without a secret"
                                )
                                await socket.send(
                                    json.dumps(
                                        {
                                            "success": False,
                                            "error": "A secret is required!",
                                        }
                                    )
                                )
                                continue
                            if msg["secret"] not in ctx.api_secrets:
                                ctx.logger.warning("Client sent invalid secret")
                                await socket.send(
                                    json.dumps(
                                        {
                                            "success": False,
                                            "error": "Invalid secret! Are you a hacker?",
                                        }
                                    )
                                )
                                continue

                        if not ctx.steam_socket:
                            ctx.logger.warning(
                                "Received command before connecting to Steam"
                            )
                            await socket.send(
                                json.dumps(
                                    {
                                        "success": False,
                                        "error": "Not connected to Steam",
                                    }
                                )
                            )
                            continue

                        id = ctx.last_message_id
                        ctx.last_message_id += 1

                        if ctx.last_message_id == MAX_MESSAGE_ID:
                            ctx.last_message_id = 0

                        ctx.message_map[id] = socket

                        if "messageId" in msg:
                            ctx.id_map[id] = msg["messageId"]

                        if "args" in msg:
                            await ctx.steam_socket.send(
                                json.dumps(
                                    {
                                        "messageId": id,
                                        "secret": ctx.rpc_secret,
                                        "command": msg["command"],
                                        "args": msg["args"],
                                    }
                                )
                            )
                        else:
                            await ctx.steam_socket.send(
                                json.dumps(
                                    {
                                        "messageId": id,
                                        "secret": ctx.rpc_secret,
                                        "command": msg["command"],
                                    }
                                )
                            )

                        ctx.logger.debug("Sent command to Steam:", message)
        except asyncio.CancelledError:
            closing = True
        finally:
            if socket == ctx.steam_socket:
                if not reconnecting and not closing:
                    reconnecting = True

                    task = asyncio.create_task(reconnect_to_steam())
                    ctx.background_tasks.add(task)
                    task.add_done_callback(ctx.background_tasks.discard)

    return handler


def make_payload(ctx: Context, port: int, rpc_secret: str, replace: bool):
    payload = ""
    try:
        with open("out/payload.template.js", "r") as file:
            for line in file.readlines():
                payload += (
                    line.replace("$PORT", str(port))
                    .replace("$SECRET", rpc_secret)
                    .replace("$REPLACE", "true" if replace else "false")
                )

            return payload
    except FileNotFoundError:
        ctx.logger.critical("Payload file not found!")
        ctx.server.close()
        return ""


async def send_payload(debugger_url: str, payload: str):
    async with websockets.connect(debugger_url) as ws:
        msg_id = 1

        command = {
            "id": msg_id,
            "method": "Runtime.evaluate",
            "params": {"expression": payload, "awaitPromise": True},
        }

        await ws.send(json.dumps(command))


def kill_previous_instances():
    pid_file = Path(tempfile.gettempdir()).joinpath("conductor.pid")

    if pid_file.exists():
        try:
            with open(pid_file, "r") as f:
                old_pid = int(f.read())
            os.kill(old_pid, signal.SIGTERM)
        except (ProcessLookupError, PermissionError, OSError):
            pass

    with open(pid_file, "w") as f:
        f.write(str(os.getpid()))

    import atexit

    atexit.register(lambda: pid_file.unlink(missing_ok=True))


async def main(ctx: Context):
    kill_previous_instances()

    await asyncio.sleep(0.1)

    ctx.rpc_secret = secrets.token_urlsafe(16)

    ctx.logger.info("Starting Conductor...")

    try:
        with open("secrets.json") as f:
            try:
                ctx.api_secrets = json.load(f)
                ctx.logger.info("Found secrets.json, turning on authentication")
            except json.JSONDecodeError:
                ctx.logger.warning("secrets.json is not valid JSON")
            except UnicodeDecodeError:
                ctx.logger.warning("secrets.json is not valid Unicode")
    except OSError:
        ctx.logger.info("secrets.json not found, turning off authentication")

    ctx.server = await serve(make_handler(ctx), "", ctx.port)

    js_context_tab: dict[str, str] = {}

    while True:
        try:
            res = requests.get("http://localhost:8080/json", timeout=1)
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
            ctx.logger.info("Connection to Steam client failed, retrying...")
            await asyncio.sleep(0.5)

    if "webSocketDebuggerUrl" not in js_context_tab:
        ctx.logger.critical("SharedJSContext has no debugger URL!")
        return

    ctx.debugger_url = js_context_tab["webSocketDebuggerUrl"]
    ctx.logger.debug("Sending payload to:", ctx.debugger_url)

    payload = make_payload(ctx, ctx.port, ctx.rpc_secret, True)

    tries = MAX_PAYLOAD_TRIES
    while True:
        try:
            await send_payload(ctx.debugger_url, payload)
            break
        except (ConnectionRefusedError, websockets.exceptions.InvalidStatus):
            ctx.logger.warning("Failed to send payload, retrying...")
            tries -= 1
            if tries == 0:
                ctx.logger.error(
                    "Failed to send payload. Check if Steam is running with remote debugging enabled."
                )
                return

    await ctx.server.serve_forever()


if __name__ == "__main__":
    ctx = Context()

    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(ColorFormatter())

    root_logger = logging.getLogger(None)
    root_logger.addHandler(handler)
    root_logger.setLevel(logging.INFO)

    for lib in ["requests", "websockets", "websockets.server"]:
        lib_logger = logging.getLogger(lib)
        lib_logger.addFilter(DowngradeInfoFilter())
        lib_logger.propagate = False

    ctx.logger = logging.getLogger(__name__)

    s = requests.Session()
    s.mount("http://", HTTPAdapter(max_retries=0))

    try:
        asyncio.run(main(ctx))
    except (asyncio.CancelledError, KeyboardInterrupt):
        ctx.closing = True
        ctx.logger.info("Goodbye!")
