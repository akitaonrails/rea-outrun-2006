# Third-party components

Everything bundled with or compiled into the package, with its licence and
source. Binary components are never committed: `packaging/package.sh`
fetches them from the URLs pinned in `packaging/*.txt` and checks each one
by SHA-256. Add a row (and a pin) before bundling anything new.

## Compiled into OutRunLauncher.exe

| Component | Licence | Source |
| --- | --- | --- |
| `sha2` and its RustCrypto dependencies | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| `windows-sys` | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `eframe`, `egui` and their dependencies (`winit`, `glutin`, `glow`, …): the settings window | MIT OR Apache-2.0 (dependencies: MIT, Apache-2.0, Zlib, BSD; see `cargo tree`) | https://github.com/emilk/egui |
| `jpeg-decoder` | MIT OR Apache-2.0 | https://github.com/image-rs/jpeg-decoder |
| Rust standard library | MIT OR Apache-2.0 | https://github.com/rust-lang/rust |
| MinGW-w64 runtime (static parts linked by the GNU target) | ZPL-2.1 / public domain | https://www.mingw-w64.org |

## Settings window artwork and fonts (compiled into OutRunLauncher.exe)

| File | Licence | Source |
| --- | --- | --- |
| `crates/or-launcher/assets/portrait.jpg`: a sunset coast with a red convertible, an illustration generated for this project (no game artwork) | ours (MIT OR Apache-2.0) | this repository |
| Barlow and Barlow Condensed (`crates/or-launcher/assets/fonts`) | SIL OFL 1.1 (`OFL.txt`) | https://github.com/jpt/barlow |

## Bundled in the package (fetched by `packaging/package.sh`)

| Folder | Component | Licence | Pin |
| --- | --- | --- | --- |
| `tweaks/` | OutRun2006Tweaks by emoose, CI build of commit `08e5efb4`, repacked as a release of this repository with its `LICENSE.md` and `SOURCE.txt` (docs/TWEAKS.md): `dinput8.dll`, the DRM-free replacement `OR2006C2C.exe`, `OutRun2006Tweaks.ini`, `OutRun2006Tweaks.lods.ini` | MIT at this commit (the project moved to GPL-3.0 at `021e4bf5`; the release also carries the source); the replacement EXE is derived from SEGA's game EXE | `packaging/tweaks.txt`, https://github.com/emoose/OutRun2006Tweaks |
| `soundtrack/` | OutRun2006Tweaks Soundtrack Fix 1.0: `14_Rush_a_Difficulty_1989.ogg.vcdiff` | no licence of its own (published with OutRun2006Tweaks, then MIT); the patch restores SEGA audio | `packaging/soundtrack.txt` |
| `soundtrack/` | xdelta3 3.1.0 (i686), shipped in the Soundtrack Fix as `xdelta3-3.1.0-i686.exe`, renamed `xdelta3.exe` | GPL-2.0 | https://github.com/jmacd/xdelta-gpl |
| `linux/` | DXVK 3.1.1, 32-bit `d3d9.dll` (used under Wine and Proton only) | zlib | `packaging/dxvk.txt`, https://github.com/doitsujin/dxvk |
| `vc/` | Microsoft Visual C++ 2015–2022 runtime 14.44.35211.0 (x86): `msvcp140.dll`, `vcruntime140.dll`, `vcomp140.dll`, deployed app-local | Microsoft Visual Studio Distributable Code | `packaging/vcredist.txt`, https://learn.microsoft.com/cpp/windows/latest-supported-vc-redist |

## Bundled in the Linux AppImage (`packaging/appimage`)

| Component | Licence | Source |
| --- | --- | --- |
| Wine 11.20 (amd64, WoW64), Kron4ek's build of the unmodified WineHQ sources; licence and source notes inside the AppImage under `usr/share/doc/wine` | LGPL-2.1-or-later | `packaging/wine.txt`, https://dl.winehq.org/wine/source/11.x/wine-11.20.tar.xz, build scripts https://github.com/Kron4ek/Wine-Builds |
| AppImage runtime (added by `appimagetool` 1.9.1) | MIT | `packaging/appimagetool.txt`, https://github.com/AppImage/type2-runtime |
| The AppImage icon `packaging/appimage/outrun-launcher.png` | ours (MIT OR Apache-2.0) | this repository |

## Not bundled

The community HD texture pack (Envido32's OR2006Sprites compilation): no
licence is stated, so the launcher links to its download page instead of
shipping it (docs/HD_TEXTURES.md).

The game itself. OutRun is a trademark of SEGA; players need their own
copy of OutRun 2006: Coast 2 Coast.
