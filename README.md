# SteamyRPC

## About

SteamyRPC is a small but powerful script that lets you control your Steam client with a WebSocket interface. Inspired by [Decky Loader](https://github.com/SteamDeckHomebrew/decky-loader).

## Installing

1. Install the [`websockets`](https://pypi.org/project/websockets/) Python package with uv or pip
2. Download a release zip and unpack anywhere you want
3. Open Steam
4. Run `server.py`

Repeat the last two steps any time you want to use SteamyRPC again.

## Connecting

You can connect to the server at `ws://[your ip]:7355`. All requests should be formatted like this:

```typescript
{
    messageId: number,
    command: string,
    args: object,
}
```

The type of `args` depends on what command you're calling. SteamyRPC will send back a WebSocket message with the message ID and return values (if any) to let you know it finished.

## Commands

### AddShortcut

Adds a shortcut (non-Steam game) to your library.

**Arguments:**

```typescript
{
    name: string, // game title
    exe: string, // path to the game executable
    launchOptions: string[], // launch options for the game
    icon: string, // path to the game icon (.jpg, .png, .exe, .ico)
}
```

**Returns**:

```typescript
{
    appId: number, // the new shortcut's app ID
}
```

### RemoveShortcut

Removes a shortcut from your library.

**Arguments:**

```typescript
{
    appId: number, // the shortcut's app ID, returned by AddShortcut
}
```
