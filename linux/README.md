# OutRun 2006: Coast 2 Coast on Linux

The launcher is a Windows program; on Linux it runs through Wine or
Proton. There are three ways to play: the AppImage, which brings its own
Wine; Steam with Proton; or `or-play` with your own Wine. Each one opens
the same settings window as on Windows.

## The AppImage

`chmod +x OutRunLauncher-<version>-x86_64.AppImage` and run it. It
carries a portable Wine and runs `or-play` (below) with it, so you need no
Wine of your own. Its Wine prefix is `~/.local/share/or-launcher/prefix`.
With [gamescope](https://github.com/ValveSoftware/gamescope) installed the
game runs inside it; without it the game runs in a normal window.

## Steam (Proton)

Add `OutRunLauncher.exe` from the unpacked zip as a non-Steam game and
force a Proton version in its compatibility settings. The launcher notices
Wine and sets up DXVK and the DLL overrides itself.

### Steam Deck

In Desktop Mode, add `OutRunLauncher.exe` the same way, or run the
AppImage. The window recommends the **Steam Deck** look. In Game Mode,
`or-play` doesn't start a gamescope of its own inside Steam's. Game Mode
hasn't been tested yet.

## Your own Wine

`or-play` starts the game through Wine inside gamescope. gamescope gives
the game a single landscape screen of the size you picked and scales it to
your monitor without changing its shape, so Wine never picks a portrait
monitor or switches your desktop's video mode.

```sh
or-play          # the settings window, then the game
or-play --play   # the game with the saved settings, no window
or-play --play --look original --scaler nearest   # launcher options for one run
or-play stop     # end the game session or-play started
or-play status   # running or idle
```

It needs `wine` and, ideally, `gamescope`. The settings window is the
launcher's own, shown through Wine before gamescope starts. On Linux it
has one more choice, the **Scaler** (how gamescope enlarges the picture):
**Smooth** (`linear`, the default), **Sharp** (`nearest`), **Even pixels**
(`pixel`) and **FSR** (`fsr`, smooth then sharpened). Everything is saved
in `launcher.cfg` inside the Wine prefix.

Options after `--play` go to the launcher for that run; `--scaler` and
`--resolution` also set gamescope's filter and size for that run only.
With `resolution = original` or `off` gamescope matches the game's
640×480; with no resolution saved (the **Original 2006** look) it uses
1920×1080.

The launcher sets the DLL overrides the game needs and copies DXVK into
its private copy of the game, so the prefix needs no winetricks. Under
Wine the game reads pads through XInput by default (Wine's
Windows.Gaming.Input loses the d-pad); the Controls tab offers the others.

`or-play` stays open until the game ends (a distrobox or AppImage session
would otherwise end the game with it), and refuses to start a second game
while one is running or still loading. The game runs in a session of its
own; `or-play stop` ends that session and what is left in the prefix, and
never another program that took over an old process number. Pads keep
working while the game's window is in the background. Inside a gamescope
session (a Steam Deck in Game Mode), or when gamescope isn't installed, it
runs the game directly.

## Environment

| Variable | Meaning | Default |
| --- | --- | --- |
| `OR_PREFIX` | the Wine prefix | `~/.local/share/or-launcher/prefix` |
| `OR_LAUNCHER` | path to `OutRunLauncher.exe` | next to `or-play`, or one folder up |
| `OR_RUN` | a wrapper that runs wine and gamescope, e.g. `distrobox enter gaming --` | none |
| `WINEDLLOVERRIDES` | your own DLL overrides; rules for the DLLs the launcher sets (`dinput8`, `d3d9`, `mscoree`, `mshtml`) are replaced by its own | none |
| `SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS`, `SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS` | pads keep working in the background | 1 (your own value wins) |

## Where things are

| What | Where |
| --- | --- |
| Settings (`launcher.cfg`) and log (`launcher.log`) | in the prefix, `drive_c/users/<you>/AppData/Local/OutRunLauncher/` |
| The launcher's copy of the game | in the prefix, `…/AppData/Local/OutRunLauncher/OutRun2006` |
| Your saves | in the prefix, `drive_c/users/<you>/Saved Games/OutRun 2006` |
| The downloaded HD pack | in the prefix, `…/AppData/Local/OutRunLauncher/hd-pack` |
| Wine prefix | `~/.local/share/or-launcher/prefix` (`OR_PREFIX`) |
| `or-play`'s log (`play.log`) and lock | `~/.config/or-launcher` |

Under Proton the same files are in Steam's `compatdata/<appid>/pfx`
instead of the prefix above.

## Troubleshooting

- **The d-pad doesn't move the menus.** Keep the Input API on Automatic
  (XInput under Wine), or try DirectInput in the Controls tab. On 8BitDo
  pads use the X-input mode switch.
- **`or-play` says a game is already running.** End it with `or-play stop`.
- **Anything else:** attach `~/.config/or-launcher/play.log` and
  `launcher.log` to the
  [bug report form](https://github.com/akitaonrails/rea-outrun-2006/issues/new?template=bug-report.yml).
