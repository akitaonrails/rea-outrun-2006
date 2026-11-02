# Project rules

This repository builds a self-contained compatibility package that runs the
user's own copy of the 2006 PC game **OutRun 2006: Coast 2 Coast** on modern
Windows and on Linux (Proton/Wine). It is not a distribution of the game.

## Non-negotiable boundaries

- Never commit, vendor, embed, or publish original executables, DLLs, media,
  archives, music, track data, or other game assets, or anything copied out
  of them. Binary components (OutRun2006Tweaks, DXVK, Wine, xdelta3, the
  VC++ runtime, the soundtrack vcdiff, the community texture pack) are
  fetched at package time from URLs and digests pinned in `packaging/*.txt`
  and are never committed.
- The user's game folder is read only. The launcher builds a separate runtime
  folder from it.
- Never commit a developer's absolute game path. Commands and code take an
  explicit path (or `OR2006_GAME_DIR` in examples).
- Put local experiments, traces and investigation output under `.work/`.
- Running the user's original game binaries is allowed; run them from a
  folder outside the repository and record notable runtime observations in
  `docs/RUNTIME_NOTES.md`.

## Product rules

- **Self-contained.** The user installs this package and points it at the
  game. Never require third-party patchers, winetricks commands, DLL
  overrides typed by hand, compatibility modes, or manual registry edits;
  whatever the game needs is bundled and configured by us.
- **Windows first, easy on Linux.** Ship Windows binaries that also run under
  Proton or Wine with no extra setup; an AppImage with a portable Wine for
  Linux users without Steam.
- **Reuse open source.** OutRun2006Tweaks (GPL-3.0) is the engine of the
  fixes: we bundle its replacement EXE and `dinput8.dll`, generate its INI
  from our settings, and never reimplement what it already patches. DXVK
  serves D3D9 on Linux. xdelta3 applies the soundtrack fix. Check licences
  before bundling and record each component with its licence and source in
  `THIRD_PARTY.md`.
- **No on-disk patching of the user's files.** Our changes go into the
  private runtime copy only. The Tweaks replacement EXE is dropped in
  verbatim; we do not byte-patch game binaries ourselves.
- The faithful, unchanged game is available as a look (Original 2006); the
  recommended default is the Coast to Coast look (owner decision, from the
  distrobox-gaming tuned configuration of 2026-07-12).

## Code and commits

- General solutions first. Settings flow one way: launcher.cfg → the two
  INI files (`outrun2006.ini`, `OutRun2006Tweaks.ini`) → the game. INI
  writing preserves every comment and unmanaged key the user may have
  edited; the launcher re-applies only the keys it owns, every run.
- Rust for our own code. Keep `unsafe` and Win32 FFI in `win32.rs`.
- Unit-test the logic that can run on the host (edition detection, INI
  writing, configuration, runtime planning, manifest sync) before each
  milestone commit; record what was checked by running the game in
  `docs/RUNTIME_NOTES.md`.
- Use Conventional Commit subjects. Commit at working milestones.
