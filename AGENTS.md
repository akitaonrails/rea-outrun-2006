# OutRun 2006 launcher — instructions for coding agents

This repository makes the player's own copy of **OutRun 2006: Coast 2
Coast** (2006, PC) run on modern Windows and Linux (Proton/Wine, AppImage)
with a settings window, sensible defaults and no manual steps. It is not a
distribution of the game. It is built on
[rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk) at the
tag in `sdk/VERSION`.

**Read `sdk/docs/STANDARDS.md` before any work; the playbooks are in
`sdk/docs/playbooks/`** (vendored from the SDK at the tag in `sdk/VERSION`).
This file holds only what is specific to OutRun.

## Always

- The game's files are read-only: never committed, copied into the repo or published.
- Everything outside this repository is the user's and read-only (game installs,
  other repos, Wine prefixes, saves, home configs, containers, the SDK checkout)
  unless the user asked for or approved that specific change: back up first,
  never delete by a broad glob, ask when unsure.
- Headless by default; open a visible window only when a check needs a person, and say so first.
- Never print, log or commit credentials.
- Push, tag, release or post only with the user's go-ahead.
- Research community fixes first (`docs/RESEARCH.md`); patch only what nobody solved.
- Never edit `sdk/`; SDK gaps and bugs go to the SDK repository as issues or PRs.
- Clean up after a release or a long session: `sdk/scripts/clean`, then `--yes` (log scratch in `.work/.task-created`).

## Owner decisions

- **2026-11-02: OutRun2006Tweaks' DRM-free replacement EXE is approved.**
  The 2006 `OR2006C2C.exe` is wrapped in disc DRM; the launcher drops the
  maintained community mod's replacement EXE into its private copy
  verbatim. We never write our own crack or byte-patch the game's EXE.
  Recorded in `THIRD_PARTY.md` too.
- **2026-11-02: unset settings mean the launcher's defaults** (the Coast to
  Coast look) for the keys it owns, documented in the `launcher.cfg`
  template. `outrun2006.ini`'s other keys (language, adapter, fog) stay the
  game's.
- **2026-11-01 (recorded): the Coast to Coast look is the recommended default**, from
  the distrobox-gaming tuned configuration of 2026-07-12; the faithful,
  unchanged game is the Original 2006 look.
- **2026-11-02: `resolution = off` under `or-play` is the game's 640×480**
  in gamescope too (`REA_ORIGINAL_WORDS="original off"`), as the launcher
  reads it.

## OutRun-specific rules

- **OutRun2006Tweaks is the engine of the fixes** (MIT at the pinned build,
  GPL-3.0 from upstream commit `021e4bf5` on): we bundle its replacement
  EXE and `dinput8.dll`, generate its INI from our settings, and never
  reimplement what it already patches. Upstream publishes current builds
  only as expiring CI artifacts, so each build we use is repacked as a
  `tweaks-<commit>` release of this repository by
  `packaging/update-tweaks.sh` (`docs/TWEAKS.md`); never edit
  `packaging/tweaks.txt` by hand.
- **The HD texture pack has no licence** and is never bundled or mirrored:
  the launcher downloads it from its own GitHub release when the player
  asks (HD Pack tab, `--download-hd-pack`; pinned in
  `packaging/hd-pack.txt`, compiled into the exe) and the game reads it in
  place (`docs/HD_TEXTURES.md`).
- **Settings flow one way:** `launcher.cfg` → `outrun2006.ini` (video mode)
  and `OutRun2006Tweaks.ini` → the game. Both INI writers keep every
  comment and unmanaged key; the launcher re-applies the keys it owns on
  every run.
- **DXVK** serves Direct3D 9 on Linux only. Its `d3d9.dll` is recorded in
  the runtime's ledger (`.or-components`) and taken out on Windows without
  bringing the game folder's `d3d9.dll` back: repacks ship SpecialK there,
  which would hijack Direct3D (`runtime.rs` never copies it either).
- **Disabled reasons stay visible.** A pad player never hovers: the SDK's
  chip rows and switches write a disabled control's reason on the window
  (from v0.4.0), so `chip_row` and `bool_row` in `window.rs` keep the
  row's description under it and never repeat the reason. Keep the
  window's palette, the sunset-coast photo and the solid caption plate.
- Example game paths use `$OR2006_GAME_DIR`.

## Layout

| Path | What |
|---|---|
| `crates/or-launcher/src/edition.rs` | Known `OR2006C2C.exe` builds (SHA-256), the unknown-edition refusal |
| `crates/or-launcher/src/config.rs` | The option table (`OPTIONS`): `launcher.cfg`, flags, `--help`, template, looks |
| `crates/or-launcher/src/form.rs`, `window.rs`, `ui.rs` | The settings window, its look cards and its headless tests |
| `crates/or-launcher/src/plan.rs` | What a launch writes, for the option tests and golden snapshots |
| `crates/or-launcher/src/tweaks.rs` | Tweaks, VC++ runtime, DXVK, both INI files, the HD pack folder |
| `crates/or-launcher/src/runtime.rs` | The runtime copy's `Policy` and the saves mirror |
| `crates/or-launcher/src/hdpack.rs`, `soundtrack.rs` | The HD pack download, the soundtrack fix (xdelta3) |
| `crates/or-launcher/src/main.rs` | Windows wiring: modes, `--check`, `launcher.log`, the game's process |
| `crates/or-launcher/tests/` | `options.rs` (every option end to end, the CLI), `snapshots.rs` and `snapshots/` |
| `linux/or-play`, `linux/tests/` | The Linux helper (sources `sdk/rea-play.sh`) and its tests |
| `packaging/` | One pin file per component, allowlist, `package.sh`, `update-tweaks.sh`, AppImage entry |
| `sdk/` | Vendored by the SDK's `scripts/sdk-sync`; never edit by hand |
| `docs/` | `RESEARCH.md`, `RUNTIME_NOTES.md`, `CONTRIBUTIONS.md`, `TWEAKS.md`, `HD_TEXTURES.md` |

## Checks (all must pass before a commit)

```sh
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --release --target i686-pc-windows-gnu --workspace --all-targets -- -D warnings
cargo deny check
for t in linux/tests/*.sh packaging/tests/*.sh; do sh "$t"; done
../rea-launcher-sdk/scripts/sdk-sync --check .
cargo build --release --locked --target i686-pc-windows-gnu
sdk/packaging/check-imports.sh target/i686-pc-windows-gnu/release/OutRunLauncher.exe
sdk/scripts/wine-test --locked --workspace     # Windows-target tests under Wine, headless
```

MinGW, lld and Wine live in the `gaming` distrobox: run the last three as
`distrobox enter gaming -- sh -c 'cd /absolute/path && …'`.

## OutRun 2006: what is specific to this game

- **Editions:** the original 2006 retail/repack `OR2006C2C.exe` (DRM-wrapped,
  `bdafa88a…`; the launcher swaps in the Tweaks EXE in its copy) and the
  Tweaks DRM-free EXE (`68ceb386…`, an install already patched; Tweaks
  0.1-0.2.5 shipped `c926e54a…`, the same EXE without the Large Address
  Aware flag). Others are refused with their SHA-256 and the
  unknown-edition issue form; a build is added only when we can verify it.
- **Renderer, input, audio:** Direct3D 9 (DXVK under Wine); input through
  Tweaks' SDL layer (WGI on Windows, XInput under Wine by default; rumble
  and impulse triggers); the game's own audio, plus the Soundtrack Fix
  patch to the player's "Rush a Difficulty".
- **Files the game writes:** `outrun2006.ini` and the saves (`License.dat`,
  records) in the runtime's save folder, mirrored with the `saves` folder
  (default `Saved Games\OutRun 2006`).
- **Community components used:** `docs/RESEARCH.md` and `THIRD_PARTY.md`.
- **Patches:** none of our own.
- **Known problems and their status:** `docs/RUNTIME_NOTES.md`.
