#!/usr/bin/env python

import asyncio
import json
import requests
import websockets
import secrets

from websockets.asyncio.server import serve

global server_socket
message_map = {}

def make_handler(server_secret: str):
    global server_socket
    global message_map

    async def handler(websocket):
        async for message in websocket:
            if message == ('init:' + server_secret):
                server_socket = websocket

                print('SteamyRPC initialized!')
                await server_socket.send(json.dumps({
                    'messageId': 0,
                    'secret': server_secret,
                    'command': 'AddShortcut',
                    'args': {
                        'name': 'Aseprite',
                        'icon': r'F:\Software\Aseprite\aseprite.exe',
                        'exe': r'F:\Software\Aseprite\aseprite.exe',
                        'launchOptions': [],
                        'startDir': r'F:\Software\Aseprite'
                    }
                }))
            else:
                if websocket == server_socket:
                    print('Received message from Steam:', message)

                    msg = json.loads(message)
                    id = msg['messageId']

                    if id in message_map:
                        msg.pop('messageId', None)
                        print('Returned value to client:', msg)
                        message_map[id].send(json.dumps(msg))

                else:
                    msg = json.loads(message)
                    message_map[id] = websocket

                    await server_socket.send(json.dumps({
                        'messageId': msg.id,
                        'secret': server_secret,
                        'command': msg.command,
                        'args': msg.args
                    }))

                    print('Forwarded command to Steam:', message)
    return handler

def make_payload(port: int, server_secret: str):
    payload = ''
    with open('client.template.js', 'r') as file:
        for line in file.readlines():
            payload += line.replace(r'{{PORT}}', str(port)).replace(r'{{SECRET}}', server_secret)
        return payload

async def send_payload(debugger_url: str, payload: str):
    async with websockets.connect(debugger_url) as ws:
        msg_id = 1

        command = {
            "id": msg_id,
            "method": "Runtime.evaluate",
            "params": {
                "expression": payload,
                "awaitPromise": True
            }
        }

        await ws.send(json.dumps(command))

async def main():
    server_secret = secrets.token_urlsafe(16)
    port = 7355

    print('Starting SteamyRPC...')

    server = await serve(make_handler(server_secret), "", port)

    tabs = requests.get('http://localhost:8080/json').json()

    for tab in tabs:
        if tab['title'] == 'SharedJSContext':
            debugger_url = tab['webSocketDebuggerUrl']
            print('Sending payload to:', debugger_url)

            payload = make_payload(port, server_secret)

            await send_payload(debugger_url, payload)
            break

    await server.serve_forever()


if __name__ == "__main__":
   asyncio.run(main())