# Third-party components

Everything bundled with or compiled into the package, with its licence and
source. Binary components are never committed: `packaging/package.sh`
fetches them from the URLs pinned in `packaging/*.txt` and checks each one
by SHA-256. Add a row (and a pin) before bundling anything new.

## Compiled into OutRunLauncher.exe

| Component | Licence | Source |
| --- | --- | --- |
| `rea-launcher`, `rea-launcher-win32`, `rea-launcher-ui`: the shared launcher SDK (runtime copy, saves, INI and settings editing, Win32 helpers, the settings window's kit) | MIT OR Apache-2.0 | https://github.com/akitaonrails/rea-launcher-sdk (the tag in `sdk/VERSION`) |
| `sha2` and its RustCrypto dependencies | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| `windows-sys` | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `eframe`, `egui` and their dependencies (`winit`, `glutin`, `glow`, …): the settings window | MIT OR Apache-2.0 (dependencies: MIT, Apache-2.0, Zlib, BSD; see `cargo tree`) | https://github.com/emilk/egui |
| `jpeg-decoder` | MIT OR Apache-2.0 | https://github.com/image-rs/jpeg-decoder |
| `zip`, `flate2`, `zlib-rs` and `crc32fast`: unpacking the HD texture pack | zip: MIT; flate2, crc32fast: MIT OR Apache-2.0; zlib-rs: Zlib | https://github.com/zip-rs/zip2, https://github.com/rust-lang/flate2-rs, https://github.com/trifectatechfoundation/zlib-rs |
| `cab` and `lzxd`: unpacking Microsoft's June 2010 DirectX 9 cabinet archives (`--download-directx`) | MIT | https://github.com/mdsteele/rust-cab |
| Rust standard library | MIT OR Apache-2.0 | https://github.com/rust-lang/rust |
| MinGW-w64 runtime (static parts linked by the GNU target) | ZPL-2.1 / public domain | https://www.mingw-w64.org |

## Images (window, package, README)

| File | Author | Licence | Source |
| --- | --- | --- | --- |
| `crates/or-launcher/assets/portrait.jpg` (compiled into OutRunLauncher.exe): a sunset coast with a red convertible, the window's hero art | an illustration generated for this project on 2026-11-01 (commit `6e34e3e`); the tool and prompt were not recorded. No game artwork, no SEGA material. **Origin to be confirmed by the owner.** | ours (MIT OR Apache-2.0), pending that confirmation | this repository |
| `packaging/appimage/outrun-launcher.png`: the AppImage and desktop icon | made for this project (commit `91c67e7`); how it was made was not recorded. **To be confirmed by the owner.** | ours (MIT OR Apache-2.0), pending that confirmation | this repository |
| `docs/images/launcher-*.jpg`: screenshots of the settings window (they show `portrait.jpg`) | this project | ours (MIT OR Apache-2.0) | this repository |
| README showcase poster and video (`showcase-poster.jpg`, `showcase-h264.mp4`): the settings window and the owner's own play-through on Linux, hosted outside the repository | the owner | screenshots and footage of the game, shown to illustrate the launcher; the game's images belong to SEGA | https://new-uploads-akitaonrails.s3.us-east-2.amazonaws.com/2026/11/02/outrun-2006/ |

## Fonts (compiled into OutRunLauncher.exe)

| File | Licence | Source |
| --- | --- | --- |
| Barlow and Barlow Condensed, bundled by `rea-launcher-ui` (`crates/rea-launcher-ui/assets/fonts` in the SDK) | SIL OFL 1.1 (`OFL.txt` there) | https://github.com/jpt/barlow |

## Bundled in the package (fetched by `packaging/package.sh`)

| Folder | Component | Licence | Pin |
| --- | --- | --- | --- |
| `tweaks/` | OutRun2006Tweaks by emoose, CI build of commit `08e5efb4`, repacked as a release of this repository with its `LICENSE.md` and `SOURCE.txt` (docs/TWEAKS.md): `dinput8.dll`, the DRM-free replacement `OR2006C2C.exe`, `OutRun2006Tweaks.ini`, `OutRun2006Tweaks.lods.ini` | MIT at this commit (the project moved to GPL-3.0 at `021e4bf5`; the release also carries the source); the replacement EXE is derived from SEGA's game EXE | `packaging/tweaks.txt`, https://github.com/emoose/OutRun2006Tweaks |
| `soundtrack/` | OutRun2006Tweaks Soundtrack Fix 1.0: `14_Rush_a_Difficulty_1989.ogg.vcdiff` | no licence of its own (published with OutRun2006Tweaks, then MIT); the patch restores SEGA audio | `packaging/soundtrack.txt` |
| `soundtrack/` | xdelta3 3.1.0 (i686), shipped in the Soundtrack Fix as `xdelta3-3.1.0-i686.exe`, renamed `xdelta3.exe` | GPL-2.0 | https://github.com/jmacd/xdelta-gpl |
| `linux/` | DXVK 3.1.1, 32-bit `d3d9.dll` (used under Wine and Proton only) | zlib (mirrored in this repository's `thirdparty` release) | `packaging/dxvk.txt`, https://github.com/doitsujin/dxvk |
| `vc/` | Microsoft Visual C++ 2015–2022 runtime 14.44.35211.0 (x86): `msvcp140.dll`, `vcruntime140.dll`, `vcomp140.dll`, deployed app-local | Microsoft Visual Studio Distributable Code (the redistributable package; mirrored in the `thirdparty` release) | `packaging/vcredist.txt`, https://learn.microsoft.com/cpp/windows/latest-supported-vc-redist |

## Bundled in the Linux AppImage (`packaging/appimage`)

| Component | Licence | Source |
| --- | --- | --- |
| Wine 11.20 (amd64, WoW64), Kron4ek's build of the unmodified WineHQ sources; licence and source notes inside the AppImage under `usr/share/doc/wine` | LGPL-2.1-or-later | `sdk/packaging/appimage/tools.pins` (rea-launcher-sdk), https://dl.winehq.org/wine/source/11.x/wine-11.20.tar.xz, build scripts https://github.com/Kron4ek/Wine-Builds |
| AppImage type2 runtime, release `20251108` (added by `appimagetool` 1.9.1) | MIT | `sdk/packaging/appimage/tools.pins`, https://github.com/AppImage/type2-runtime/releases/tag/20251108 |
| The AppImage icon `packaging/appimage/outrun-launcher.png` | ours (see Images) | this repository |

## Owner decisions

- **2026-11-02:** OutRun2006Tweaks' DRM-free replacement `OR2006C2C.exe`
  (derived from SEGA's EXE, published by the maintained OutRun2006Tweaks
  project) is bundled and dropped into the launcher's private copy of the
  game. Approved by the owner on 2026-11-02 as a maintained community fix
  (rea-launcher-sdk `docs/STANDARDS.md`, "DRM"); also in `AGENTS.md`.

## Not bundled

The community HD texture pack (Envido32's OR2006Sprites compilation): no
licence is stated, so it is never shipped. The launcher downloads it from
the pack's own GitHub release when the player asks for it, pinned by
`packaging/hd-pack.txt` (docs/HD_TEXTURES.md).

Microsoft's June 2010 DirectX 9 redistributable (`d3dx9_29.dll`, …):
Microsoft's DirectX End-User Runtime licence does not allow redistributing
the unpacked DLLs inside our zip. On Windows the launcher checks whether
`d3dx9_29.dll` is present, shows a warning in the settings window when it
is missing, and offers a one-click download from Microsoft's official
redistributable into `%LOCALAPPDATA%\OutRunLauncher\directx`.

The game itself. OutRun is a trademark of SEGA; players need their own
copy of OutRun 2006: Coast 2 Coast.
