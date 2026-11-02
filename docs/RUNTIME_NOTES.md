# Runtime notes

What was checked by running the game, newest first. Paths use
`$OR2006_GAME_DIR` for the player's game folder.

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
