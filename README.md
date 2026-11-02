# OutRun 2006: Coast 2 Coast — play it today

> **Work in progress.** Built and unit-tested; play-testing is under way on
> Linux and still needed on native Windows. Reports and pull requests are
> welcome at <https://github.com/akitaonrails/rea-outrun-2006/issues>.

Play your own copy of the 2006 PC game **OutRun 2006: Coast 2 Coast** on a
modern computer: Windows 10/11, or Linux through Steam (Proton), Wine, or
the AppImage (which brings its own Wine).

Point the launcher at your installed game and it does the rest. You don't
need to install mods, patchers, winetricks, DLL overrides, compatibility
modes or registry edits. Your game folder is never changed: the launcher
works on its own private copy.

This project contains **no game files**. You need your own copy of the game.

## What it does

The fixes come from [OutRun2006Tweaks](https://github.com/emoose/OutRun2006Tweaks),
the community mod that makes the PC release work well today. The launcher
bundles it, configures it from one settings window, and keeps it in step
with your choices on every start.

**Runs on today's computers**

- Tweaks' DRM-free replacement EXE, so the 2006 disc protection doesn't stop
  the game on Windows 10/11 or Wine.
- The Visual C++ runtime the mod needs is deployed next to the game; no
  installer runs.
- On Linux, DXVK serves the game's Direct3D 9 through Vulkan, wired by the
  launcher. The prefix needs no winetricks.
- Your saves (License.dat and the SaveGame folder) live in a folder you
  choose, and are copied in before the game starts and back when it ends.

**Picture**

- Any resolution up to 4K, as a borderless window (or exclusive fullscreen).
- Frame-rate cap with interpolation above 60 fps, and on, off or half vsync.
- Longer draw distance, sharper car reflections, anisotropic filtering, the
  console versions' sky glow, the Xbox brightness, and the hi-def passenger
  models the PC release shipped unused.
- Optional community HD texture pack, or any texture folder of your own.

**Gamepads**

- Tweaks' modern input system: hot-plug, adjustable steering deadzone,
  rumble with a strength setting, and Xbox Series impulse triggers.

**Extras**

- CD switcher: change the music mid-race (Back = next track, RS+Back =
  previous), with optional shuffle.
- Skip the boot logos, free drive without the countdown, the horn whenever
  you like, character selection, OutRun miles on the menu.
- Restores the missing first two seconds of "Rush a Difficulty" by patching
  the launcher's copy of the song. No music is distributed.

**One-click looks**

- **Coast to Coast** (recommended): your screen's resolution, the tuned set.
- **Original 2006**: the unchanged game.
- **Steam Deck**: 1280×800, tuned for the Deck.

## 1. The game you need

**OutRun 2006: Coast 2 Coast for Windows PC**, installed. The launcher
recognises the original 2006 `OR2006C2C.exe` and a folder already patched
with OutRun2006Tweaks' replacement EXE. If yours is a different version it
says so and shows the file's SHA-256; please report it with where your copy
comes from.

## 2. Windows

1. Unpack `OutRunLauncher-<version>.zip` anywhere and keep its folders
   (`tweaks`, `vc`, `linux`, `soundtrack`, `textures-pack`) next to
   `OutRunLauncher.exe`.
2. Run `OutRunLauncher.exe`, choose the folder with your `OR2006C2C.exe`,
   pick a look, and press Play.

## 3. Linux

- **AppImage** (no Wine needed): make `OutRunLauncher-x86_64.AppImage`
  executable and run it. If [gamescope](https://github.com/ValveSoftware/gamescope)
  is installed, the game runs inside it at the size you chose.
- **Steam**: add `OutRunLauncher.exe` as a non-Steam game and force a
  Proton version.
- **Your own Wine**: run `linux/or-play` from the unpacked zip. See
  [linux/README.md](linux/README.md).

## Settings

Everything is in one commented file, `launcher.cfg`, under
`%LOCALAPPDATA%\OutRunLauncher` (or next to `OutRunLauncher.exe` for a
portable install). The settings window edits the same file. The launcher
writes the game's `outrun2006.ini` and `OutRun2006Tweaks.ini` from it on
every start, and keeps anything else you add to those files.

`OutRunLauncher --help` lists the command-line options.

## Building

```sh
cargo test                                            # host unit tests
cargo build --release --target i686-pc-windows-gnu    # needs mingw-w64 (i686)
sh linux/tests/gpu-env.sh && sh linux/tests/scaler.sh
packaging/package.sh dist                             # fetches the pinned components
packaging/appimage/build.sh dist/OutRunLauncher dist  # the AppImage
```

Third-party components are downloaded at package time from the URLs and
SHA-256 digests in `packaging/*.txt`; see [THIRD_PARTY.md](THIRD_PARTY.md).

## Licence

This project's code is MIT OR Apache-2.0. Bundled components keep their own
licences ([THIRD_PARTY.md](THIRD_PARTY.md)). OutRun is a trademark of SEGA;
this project is not affiliated with or endorsed by SEGA.
