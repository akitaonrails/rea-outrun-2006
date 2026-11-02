# Project rules

The binding standards for every rea launcher are the SDK's
[`docs/STANDARDS.md`](https://github.com/akitaonrails/rea-launcher-sdk/blob/v0.2.0/docs/STANDARDS.md);
read them before any work. This file repeats the non-negotiables in
OutRun's words and adds what is specific to this game.

This repository builds a self-contained compatibility package that runs the
user's own copy of the 2006 PC game **OutRun 2006: Coast 2 Coast** on modern
Windows and on Linux (Proton/Wine). It is not a distribution of the game.

## Non-negotiable boundaries

- Never commit, vendor, embed, or publish original executables, DLLs, media,
  archives, music, track data, or other game assets, or anything copied out
  of them. Binary components (OutRun2006Tweaks, DXVK, Wine, xdelta3, the
  VC++ runtime, the soundtrack vcdiff) are fetched at package time from
  URLs and digests pinned in `packaging/mirrored.txt` and
  `packaging/*.txt` and are never committed.
  OutRun2006Tweaks builds are repacked as `tweaks-<commit>` releases of
  this repository by `packaging/update-tweaks.sh` (docs/TWEAKS.md).
- The community HD texture pack (no licence) is never bundled: the
  launcher downloads it from its own GitHub release when the player asks
  (pinned in `packaging/hd-pack.txt`) and the game reads it in place
  (docs/HD_TEXTURES.md).
- The user's game folder is read only. The launcher builds a separate runtime
  folder from it.
- Never commit a developer's absolute game path. Commands and code take an
  explicit path (or `OR2006_GAME_DIR` in examples).
- Put local experiments, traces and investigation output under `.work/`.
- Running the user's original game binaries is allowed; run them from a
  folder outside the repository and record notable runtime observations in
  `docs/RUNTIME_NOTES.md`.

## Owner decisions

- **2026-11-02: OutRun2006Tweaks' DRM-free replacement EXE is approved.**
  The 2006 `OR2006C2C.exe` is wrapped in disc DRM; the launcher drops the
  maintained community mod's replacement EXE into its private copy
  verbatim. We never write our own crack or byte-patch the game's EXE.
  Recorded in `THIRD_PARTY.md` too.
- Unset settings mean the launcher's defaults (the Coast to Coast look)
  for the keys it owns, documented in the `launcher.cfg` template.
  `outrun2006.ini`'s other keys (language, adapter, fog) stay the game's.

## Product rules

- **Self-contained.** The user installs this package and points it at the
  game. Never require third-party patchers, winetricks commands, DLL
  overrides typed by hand, compatibility modes, or manual registry edits;
  whatever the game needs is bundled and configured by us.
- **Windows first, easy on Linux.** Ship Windows binaries that also run under
  Proton or Wine with no extra setup; an AppImage with a portable Wine for
  Linux users without Steam.
- **Reuse open source.** OutRun2006Tweaks (MIT at the pinned build, GPL-3.0
  from upstream commit `021e4bf5` on) is the engine of the
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
- Rust for our own code. Win32 calls go through `rea-launcher-win32`
  (WinHTTP downloads included); the launcher has no `unsafe` of its own.
- Unit-test the logic that can run on the host (edition detection, INI
  writing, configuration, runtime planning, manifest sync) before each
  milestone commit; record what was checked by running the game in
  `docs/RUNTIME_NOTES.md`.
- Use Conventional Commit subjects. Commit at working milestones.

## How to work here (the SDK's playbooks)

- **Research community fixes first** (PCGamingWiki, GitHub, mod sites)
  and record them in `docs/RESEARCH.md`; reuse what exists
  ([community components](https://github.com/akitaonrails/rea-launcher-sdk/blob/v0.2.0/docs/playbooks/community-components.md)).
- **Settings window changes go through a design skill** (`frontend-design`,
  then a `web-design-guidelines` review), keep OutRun's palette and art,
  and pass the accessibility rules: 3:1 control borders, body text ≥ 13 pt,
  nothing small on the photo, a visible focus ring, a reason on every
  disabled control (`window.rs` tests the palette with
  `theme::contrast_problems`).
- **GUI/CLI parity:** every option is one row of `config::OPTIONS` (key,
  flag, values, label, description). Window rows take their words from it;
  the parity test fails when the window and the table differ. Every option
  needs an end-to-end test (`tests/options.rs`: flag and key → the game
  value through `plan::plan`) and a headless GUI test (`window.rs`,
  `egui_kittest`).
- **Golden snapshots** (`tests/snapshots/`) hold `--help`, the template and
  what each look writes. A diff must match an intended change or an SDK
  migration item; never regenerate one blindly (`UPDATE_SNAPSHOTS=1`).
- **Headless testing:** host tests, then `sdk/scripts/wine-test --locked
  --workspace` for the Windows target (the GUI tests run there too).
- **Binary patching:** none here. Should a patch ever be needed, follow
  [binary-patching](https://github.com/akitaonrails/rea-launcher-sdk/blob/v0.2.0/docs/playbooks/binary-patching.md)
  (`rea_launcher::patch` tables, SHA-256 identified builds only).
- **Contribute back:** fixes we found go to `docs/CONTRIBUTIONS.md` as
  drafts ([contribute-back](https://github.com/akitaonrails/rea-launcher-sdk/blob/v0.2.0/docs/playbooks/contribute-back.md));
  the owner posts them.
- **Credentials** (e.g. `~/.config/rea/pcgamingwiki.txt`) are never
  printed, logged, committed, passed as arguments or handed to other
  agents.
- **Public actions** (push, tags, releases, issues, wiki edits) need the
  owner's go-ahead.

## Shared SDK

- Generic launcher code lives in
  [rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk):
  the `rea-launcher` crate (runtime copy, saves mirror, INI and
  `launcher.cfg` editing, identification, Wine overrides, display, pad,
  log, option tables, downloads), `rea-launcher-win32` (dialogs,
  registry, XInput, displays, Wine DPI, console, WinHTTP) and
  `rea-launcher-ui` (theme, metrics, fonts, widgets, download row, window
  sizing, pad navigation, the `testing` harness), as git dependencies; and, vendored under `sdk/`, the
  play library `rea-play.sh` that `linux/or-play` sources and the
  packaging scripts (pins, AppImage, allowlist, release notes). CI runs
  the SDK's reusable workflows.
- OutRun's knowledge stays here: the runtime `Policy`, the edition table,
  the option table, the two INI generators, the Tweaks, DXVK, HD pack and
  soundtrack steps, the window's tabs, palette and art, or-play's
  settings.
- Never fork SDK code into this repository or edit `sdk/` by hand. When
  the SDK lacks something or has a bug, keep local code for that piece and
  propose the change upstream.
- To bump, follow the SDK's
  [sdk-change playbook](https://github.com/akitaonrails/rea-launcher-sdk/blob/v0.2.0/docs/playbooks/sdk-change.md):
  tag in the root `Cargo.toml` (all three crates), `.github/workflows/*.yml`
  `@vX.Y.Z`, `scripts/sdk-sync .` from an SDK checkout at the same tag,
  every Migration item, then the snapshots must still match
  (`scripts/sdk-sync --check .` is what CI runs).
- Issues and PRs about shared code go to the SDK, not here (README,
  "Contributing").
- Tests and probes run headless by default: the Windows-target tests under
  Wine with the SDK's `scripts/wine-test` (no display, crash dialog off),
  real-game runs in `gamescope --backend headless`. Open a visible window
  only deliberately.
