# Linux helpers

## `or-play`

Starts the game through Wine inside [gamescope](https://github.com/ValveSoftware/gamescope).
gamescope gives the game a single landscape screen of the size you picked
and scales it to your monitor without changing its shape, so Wine never
picks a portrait monitor or switches your desktop's video mode.

```sh
or-play          # the settings window, then the game
or-play --play   # the game with the saved settings, no window
or-play stop     # end the game
or-play status   # running or idle
```

The settings window is the launcher's own (the same as on Windows), shown
through Wine before gamescope starts. On Linux it has one more choice, the
scaler (how gamescope enlarges the picture): Smooth (`linear`, the default),
Sharp (`nearest`), Even pixels (`pixel`) and FSR (`fsr`, smooth then
sharpened). Everything is saved in `launcher.cfg` inside the Wine prefix.
Without a saved resolution (the Original 2006 look), gamescope uses 1920×1080.

The launcher sets the Wine DLL overrides the game needs (the mod's
`dinput8.dll` and DXVK's `d3d9.dll` load as native) and copies DXVK into its
private copy of the game, so the prefix needs no winetricks.

It needs `wine` and `gamescope`. Optional environment variables:

| Variable | Meaning | Default |
| --- | --- | --- |
| `OR_PREFIX` | the Wine prefix | `~/.local/share/or-launcher/prefix` |
| `OR_LAUNCHER` | path to `OutRunLauncher.exe` | next to `or-play`, or one folder up |
| `OR_RUN` | a wrapper that runs wine and gamescope, e.g. `distrobox enter gaming --` | none |

`or-play` refuses to start a second game while one is running or still
loading. Inside a gamescope session (a Steam Deck in Game Mode), or when
gamescope isn't installed, it runs the game directly.

## Steam / Proton

Add `OutRunLauncher.exe` as a non-Steam game and force a Proton version in
its compatibility settings. The launcher detects Wine and wires DXVK and the
DLL overrides itself.

## AppImage

`packaging/appimage/build.sh <unpacked zip folder> <output folder>` builds
`OutRunLauncher-x86_64.AppImage`: the launcher with its components,
`or-play` and a portable Wine (Kron4ek's 11.20 WoW64 build, checked by
SHA-256), so players need no Wine. Its `AppRun` puts that Wine first on the
`PATH` and runs `or-play`, which waits for the game (`OR_WAIT`) because the
AppImage is unmounted when it returns. CI builds it with every package.
