# Runtime notes

What was checked by running the game, newest first. Paths use
`$OR2006_GAME_DIR` for the player's game folder.

## Unreleased — to check by running the game

Changes made after v0.1.0 from the cross-launcher audit, covered by unit
tests but not yet seen with the game:

- Saves sync (`.or-saves` in the runtime folder): kill the game mid-session
  after a save, start again, and check the save reaches the saves folder;
  then put an older backup in the saves folder and check the game loads it.
- Settings window under Wine on a 4K screen: created at 1.5x from the
  start (no resize after creation). Check it opens at full size and fully
  drawn under XWayland with fractional scaling, and still fits and centres
  on Windows at 150 % scaling. Also with Wine's DPI set to 144 and 192
  (`winecfg` → Graphics): the window must fit the screen, about 1.5x.
- `or-play`: the game now runs in a session of its own (pid + start time
  in the lock). Check `or-play stop` from another terminal ends the game,
  gamescope and Wine, through `OR_RUN="distrobox enter gaming --"` too, and
  that `or-play` then returns.
- gamescope now gets the race size as its output size too (`-W/-H` as well
  as `-w/-h`). Check the window still fills a 4K monitor with a 1080p race
  and the chosen scaler (FSR, nearest) still applies.
- `SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS=1` and
  `SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1` are exported for the game and
  the settings window (OutRun2006Tweaks reads pads through SDL). Check the
  pad in menus and in a race with XInput and WGI, and that nothing reads
  it twice.

## 2026-11-02 — the owner's play-test before v0.1.0

The owner played through `linux/or-play` (gaming distrobox, Wine 11.21,
gamescope, 3840×2160, HD pack on, an Xbox-style pad):

- `or-play` first failed to start the game twice: its inner script killed
  Wine when `OR2006C2C.exe` wasn't up 3 s after start (Wine starting in a
  fresh gamescope takes longer), and the backgrounded game then died with
  the `distrobox enter` session. Fixed: or-play waits for the launcher, in
  the foreground.
- With Input API **XInput** the d-pad moves the menus; with
  Windows.Gaming.Input (Wine's) it did nothing and only the stick worked.
  XInput is now the default under Wine and Proton.
- Racing, A/B in menus, the stick: fine. "Working fine now" on the final
  build.

## 2026-11-01 — HD pack download, settings window, option audit

**HD pack download under Wine (WinHTTP).** `OutRunLauncher.exe
--download-hd-pack` in the `gaming` container's prefix (Wine 11.21)
downloaded the pinned 326 MB release from GitHub over HTTPS (WinHTTP
followed the redirect to GitHub's file servers), matched its SHA-256,
unpacked 3.2 GB (3,168,683,238 bytes) into `%LOCALAPPDATA%\OutRunLauncher\hd-pack\OR2-HD-GUI-v0.26.09a`,
removed the zip and wrote `hd_textures = on` plus the folder into
launcher.cfg. Not yet checked: the game drawing the pack's sprites in a
race, and the download from the window's HD Pack tab (same code path).

**Settings window under Wine.** Renders with the lld build (the GNU ld
2.47 crash is gone). At 880×640 the Picture tab ran under the footer and
the Play button; the window is now 880×760 (fits a 1280×800 Steam Deck)
and tabs scroll if they still overflow; the HD pack moved to its own tab.

**Gamepad in the menus (reported by the owner).** With
`InputBackend = 0` (Windows.Gaming.Input) and the then-default
`ControllerHotPlug = true`, A and B worked in the main menu but the d-pad
did nothing and the left stick moved the selection only sometimes; driving
worked. The owner's other agents were using the same pad at the time, which
may have interfered. Since then hot-plug is always off (it only drives the
legacy input path and rescans DirectInput every 2 s). To check next: the
menus with each Input API (WGI, XInput, DInput, RawInput) under Wine, one
pad, nothing else reading it. An automated check with a virtual uinput
Xbox 360 pad and headless gamescope didn't work: headless gamescope's
screenshots don't include the game's DXVK (Vulkan) surface.

**Option audit** against the Tweaks source at `08e5efb4` (see the commit
`fix(launcher): options audited against OutRun2006Tweaks`): character
select (can crash the game), hot-plug (ignored by the new input) and half
vsync (invalid for a borderless D3D9 window) removed; anti-aliasing is the
game's MSAA; the CD switcher's previous track is Z only with the new input;
`DX/FOG = 1` (the game's default); the in-game overlay's
`OutRun2006Tweaks.user.ini` no longer overrides the launcher's keys. A fog
0 vs 1 side-by-side wasn't captured; to do.

## 2026-11-01 — first end-to-end run on Linux (or-play, Wine 11.21)

Setup: the `gaming` distrobox (Arch, Wine 11.21, gamescope, NVIDIA RTX,
`/dev/ntsync` present), a fresh prefix at
`~/.local/share/or-launcher/prefix`, the package assembled by
`packaging/package.sh` (no texture pack), `launcher.cfg` with only
`game = Z:\…\$OR2006_GAME_DIR` and `resolution = 1920x1080`, started with
`linux/or-play --play`.

The game folder is a repack already patched by an older setup:
`OR2006C2C.exe` is OutRun2006Tweaks' replacement EXE (SHA-256
`68ceb386…6134c3`, recognised as "OutRun 2006 + OutRun2006Tweaks EXE"),
and it holds disabled SpecialK `d3d9.dll`/`d3d9.ini` copies and FXT tools.
It was only read.

**Crash found and fixed: broken kernel32 imports with GNU ld 2.47.**
The first run stopped right after the saves step with
`Unhandled page fault … at address 00449E24`. A symbol build under
`winedbg` put the fault in `win32::is_wine`: its `GetModuleHandleW` slot in
the import address table belonged to no import descriptor, so the loader
never filled it and the call jumped to the slot's unpatched hint/name RVA
(`0x448e24`, the `GetModuleHandleW` name entry). The missing descriptor was
windows-sys 0.61's raw-dylib `kernel32.dll` group (`CloseHandle`,
`GetModuleHandleA/W`, `LoadLibraryW`); the standard library's own
`kernel32.dll` import survived. GNU ld 2.47 (mingw-w64-binutils
2.47.20260726) keeps one descriptor per DLL name. Linking with lld keeps
both. `.cargo/config.toml` now links the Windows targets with
`-fuse-ld=lld`; CI installs `lld`. Any Windows build of this launcher made
with ld 2.47 crashes at start on every system, Wine or not.

**Second run, with lld:** every step logged and worked:

- runtime copy: 3,562 files on the first run, 0 on the next (manifest sync)
- saves: `Saved Games\OutRun 2006` created from the game's SaveGame
  folder, then mirrored in
- `tweaks: dinput8.dll installed`, `tweaks: OR2006C2C.exe installed`
- `vc++ runtime:` msvcp140, vcruntime140, vcomp140 installed app-local
- `dxvk: installed` (DXVK 3.1.1 x32 `d3d9.dll` in the runtime copy)
- `soundtrack: Rush a Difficulty restored`: the bundled `xdelta3.exe` ran
  under Wine; the song went from 3,055,933 to 3,447,457 bytes, the
  original kept in `Sound/Backup`
- `wine: controller baseline written`, DLL overrides
  `dinput8=n,b;d3d9=n,b;mscoree,mshtml=d`

The game started inside gamescope at 1920×1080 (DXVK swapchain 1920x1080,
FIFO). `OutRun2006Tweaks.log`: every hook applied (texture replacement, UI
scaling, new input, course replacement, D3D hooks), NVIDIA SSAA enabled
for transparency supersampling, sky glow buffers at 480×270, and the
controller found by the new input system (`onControllerAdded`). A
screenshot showed a race in 16:9 with Tweaks' widescreen HUD.

Harmless log lines: `Overlay::settings_read - INI read failed` (no
`OutRun2006Tweaks.overlay.ini` until the overlay saves one), `Segoe UI
unavailable` (the overlay falls back to its own font), `UPnP: no devices`,
DXVK's `Unhandled render state 176` / `Transparency supersampling (SSAA)
is unsupported` (Tweaks then uses its NVIDIA path).

Not yet checked by playing: the settings window under Wine and on
Windows, a full race with the CD switcher and rumble, vsync and frame
pacing above 60 fps, the HD texture pack, native Windows 10/11, Proton,
and the AppImage's bundled Wine.
