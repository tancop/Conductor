# Steam Client Internals

This page has all the info I learned about the Steam client from reverse engineering it, together with a not exactly detailed explanation about how Conductor works.

## Technologies

Steam uses React in CEF (Chromium Executable Framework, like Electron but faster) for the UI and native C++ for everything else. The frontend uses MobX for state management. I wouldn't use that library at gunpoint but in this case it's useful - some RPC commands read cached data from store internals so we don't waste time recomputing it.

Just like the rest of us, Steam always keeps a lot of tabs open. 17 to be specific, or 21 (savage!) in Big Picture mode. The most important one is `SharedJSContext` used to control the whole thing and interface with native code. This is the first tab to open when the client starts and the last one to close. It stays open for almost the whole time while Steam is running and only ever closes (for a couple seconds, then restarts) when switching between desktop mode and Big Picture.

## Console

When you open Steam in dev mode (with the `-dev` command line option) you can see a console tab to the right of your username. This is the native console where Steam prints out debug messages. It's useful for debugging but doesn't let you receive messages from outside. Instead of that, Conductor uses the CEF remote debugging API to inject its payload into the JavaScript console and open a WebSocket server inside Steam's `SharedJSContext`. This API is exposed at `localhost:8080` when dev mode is on or Steam finds a file named `.cef-enable-remote-debugging` in its main folder. When Steam switches modes, the context tab closes so Conductor needs to reconnect and inject the payload again.

If you want to access the JS console manually, you got two options:

1. Open Steam in dev mode and press F12 for dev tools.
2. Open `chrome://inspect` (sorry Firefox users), add `localhost:8080` to the inspect target list with `Configure...` and wait until a list of tab names shows up. Scroll down to the bottom until you find `SharedJSContext`. Then click `inspect` on that entry. If you see two tabs with the same name click on the one with `pause` in its command list, the other one will 404.

The JS console lets you run code inside Steam with access to all the internal APIs. It's the only way to control the UI and do things the native console doesn't support, like running non-Steam games.

## Global Properties

The `window` object has some important properties used to store data and communicate with the backend.

### `SteamClient`

This object is a namespace for native calls. It's divided into sub namespaces like `Apps`, `UI` and `System`. These contain the actual functions implemented in native code. You can see the names for all of them (Steam version 1743554648) in `api-dump.json` but there's no documentation or type info, try them all out if you want! If Valve ever adds or removes functions you can dump the new API like this:

```javascript
let api = Object.fromEntries(Object.keys(SteamClient).map(key => [key, Object.keys(SteamClient[key])]));

console.log(JSON.stringify(api));
```

### `appStore.allApps`

This is an array with all the apps in your library. They're instances of a private class with no stable name, so don't try to make new ones. The most important parts are:

- `app_type` (number): the [app type](#app-types)
- `appid` (number): the app's ID, obviously
- `installed` (boolean | undefined): you know what this is
- `display_name` (string): normal app name
- `sort_as` (string): internal name used for sorting, lower case with no special characters
- `gameid` (string): internal ID used to run or terminate the app, equal to `appid.toString()` for Steam apps but not for shortcuts
- `m_setStoreCategories` (Set\<number>): store category IDs for the app. These are the top level tags like Action or Role Playing.
- `m_setStoreTags` (Set\<number>): store tag IDs for the app

### `appStore.m_mapApps`

This is a map that lets you get the app object for a specific ID, or all the IDs in your library like this:

```javascript
let cs2 = appStore.m_mapApps.data_.get(730).value_; // app object for Counter-Strike 2

let libraryIds = appStore.m_mapApps.data_.keys(); // your whole library
```

### `appStore.GetLocalizationForStoreTag(tagId: number)`

Returns the localized name for a tag ID.

### `cm`

Connection manager. The only interesting properties are `persona_name` (your display name) and `steamid`, an object that lets you get your account ID with the method `GetAccountID()`. Account IDs are not the same as Steam IDs, this is the shorter version used to name your userdata folder.

### `App.GetCurrentUser()`

Returns an object with personal info about you, like your real account name (`strAccountName`) and Steam ID (`strSteamId`).

## App Types

Every app object has a property `app_type: number` that tells Steam how to handle it. I found 8 different app types in my library:

- 1: games
- 2: software including SFM and VR drivers
- 4: tools including dedicated servers, mod tools, Proton, Steamworks redist and the delisted Half-Life 2 episodes
- 8: demos
- 256: Steam social features (news and game notes)
- 8192: soundtracks
- 65536: playtests downloaded as a separate app (NOT as a closed beta for the main app)
- 1073741824: shortcuts (non-Steam games)

You can list the types in your library like this, using the MobX store at `window.appStore`:

```javascript
let apps = window.appStore.m_mapApps.data_;

let types = apps.entries().reduce((set, [_, game]) => set.add(game.value_.app_type), new Set());

for (const type of types.keys()) {
    console.log(`${type}:`);
    for (const app of apps.entries().filter(([_, game]) => game.value_.app_type === type)) {
        console.log(app);
    }
}
```

If you find a new one let me know!
