# Changelog

All notable changes to this project. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- Under Linux (or-play, the AppImage) and Proton, an error while starting
  the game, like a wrong game folder or an EXE the launcher doesn't know,
  now shows a message. Before, it went only to the log, and it looked as
  if the game never started.
- If the launcher was closed while the game kept running, the next start
  copied files under the running game and started a second one. It now
  says the game is already running.
- On a new Wine prefix the settings window opens on the first start, not
  only on the second.
- A greyed-out setting says why once, under the setting, and its own
  description stays visible. It used to show the reason twice and hide the
  description.
- A second start finds only this launcher's window. A folder window named
  "OutRun 2006" could be taken for it.

### Changed

- A start refused because the launcher or the game is already running
  exits with code 5 (it was 0, or 3 with `--settings`). or-play says
  "nothing started".
- The bug report form asks what worked (settings window, game, pad, sound,
  quitting).
- Updated rea-launcher-sdk to v0.4.1.

## [0.2.1] - 2026-11-02

### Fixed

- The --help golden snapshot no longer contains the version, which made
  the v0.2.0 release build fail; v0.2.0 was tagged but never published,
  so this release carries everything listed under 0.2.0.

## [0.2.0] - 2026-11-02

### Added

- One launcher at a time. Starting it again while it is open brings its
  window forward; while the game runs it says so ("OutRun 2006 or its
  launcher is already running…") instead of starting a second game from the same
  copy. `--check` still runs alongside; `or-play --settings` exits with 3
  (nothing started).
- One option table (`config::OPTIONS`) declares every setting once: its
  `launcher.cfg` key, flag, values, label and description. `--help`, the
  commented `launcher.cfg` written on first run, the file and flag parsing
  and the window's labels and notes all come from it, and a test fails
  when the window and the command line differ.
- `--look original|coast_to_coast|steam_deck` (and `look =` in
  `launcher.cfg`) sets every value of a look at once; settings given on
  their own win over it.
- `--scaler` for one run (Linux): `or-play --play --scaler fsr` passes it
  to gamescope and to the launcher.
- An "Exclusive fullscreen" switch in the Picture tab (it was a
  `launcher.cfg` setting only).
- `--check` also checks the bundled `tweaks`, `vc`, `soundtrack` and (under
  Wine) `linux/d3d9.dll` files, that the runtime folder is writable and
  apart from the game, and lists `launcher.cfg` warnings; the report goes
  to `launcher.log` and the terminal (a dialog only when started without
  one). Exit code 1 when something would stop the game.
- `--help` and error messages print to the terminal the launcher was
  started from.
- The settings window runs headless in tests on any host
  (`egui_kittest`): every tab, label, description, disabled reason and
  Play are tested, with hostile paths (`José/Jogos d'Ação`).
- Golden snapshots (`crates/or-launcher/tests/snapshots/`): `--help`, the
  `launcher.cfg` template, and what each look and three option
  combinations write (both INI files, environment, winebus values,
  runtime components, gamescope filter), on Windows and Wine.
- End-to-end option tests: every option's flag and `launcher.cfg` key
  reach the same setting and the INI key, environment or gamescope filter
  it changes (`plan::plan`, what a launch writes); `--help`, every bad
  value's message, a `launcher.cfg` written by v0.1.0, and runtime and
  saves folders with spaces, apostrophes and accents.

- `docs/RESEARCH.md` (the community projects this launcher builds on and
  why), `docs/CONTRIBUTIONS.md` (the Wine d-pad finding drafted for
  PCGamingWiki and OutRun2006Tweaks), a bug report form asking for
  `launcher.log` and the EXE's SHA-256, a README uninstall section and a
  Contributing section (game issues here, shared code at rea-launcher-sdk).
- DXVK and the VC++ redistributable are mirrored in this repository's
  `thirdparty` release (`packaging/dxvk.txt`, `packaging/vcredist.txt`,
  `mirror` workflow) as
  fallbacks for the package build.
- The owner's decision of 2026-11-02 approving OutRun2006Tweaks'
  DRM-free replacement EXE is recorded in `AGENTS.md` and
  `THIRD_PARTY.md`.

### Changed

- Built on rea-launcher-sdk v0.3.3.
- The issue forms are the rea launchers' shared pair: a bug report and an
  unknown-edition form (with OutRun's questions about repacks and
  `play.log`). An unknown `OR2006C2C.exe` links to the unknown-edition
  form, and `--check` links to the bug report when something is wrong.
- A weekly `sdk-update` workflow opens a pull request when
  rea-launcher-sdk has a new tag, with its Migration items listed.
- Updated rea-launcher-sdk to v0.3.2: files that already hold the
  right bytes are no longer rewritten each start, and `--flag=value` works
  again.
- Updated rea-launcher-sdk to v0.3.0. Screen readers now name the folder
  fields by their heading ("Game folder", "Saves", "Pack folder") and read
  the hint as their description.
- DXVK's `d3d9.dll` in the runtime is recorded in the SDK's component
  ledger (`.or-components`) instead of the `.or-dxvk` marker, which an
  existing runtime has converted on its next run. On Windows it is still
  taken out without bringing back a repack's SpecialK `d3d9.dll`; a
  `d3d9.dll` that isn't the launcher's is now left alone.
- `or-play --play --resolution 1280x720` sizes gamescope for that run, as
  `--scaler` already chose its filter (the SDK's one-run helper flags
  replace or-play's own `--scaler` hook); `resolution = off` runs
  gamescope at the game's 640×480, as the launcher reads it.
- The DXVK and VC++ pins are one file each (`packaging/dxvk.txt`,
  `packaging/vcredist.txt`) like the others; the SDK's `mirror` workflow
  reads them there, so `packaging/mirrored.txt` is gone.
- The frame rate, draw distance, anisotropy and deadzone rows are the
  SDK's option rows (Range steps as chips); they look and behave as
  before. The window's tests use the SDK's `harness_sized`.
- Old spellings (`fps_limit = unlimited`, `anisotropy = off`,
  `reflections = 128`) and the removed `character_select` and `hot_plug`
  are read by the SDK's option table (value aliases, removed keys) instead
  of the launcher's own code. Saving from the window comments out a
  removed key, and `--character-select`/`--hot-plug` are now taken and
  ignored like the file's keys (they were refused). `--help` shows
  `--fps-limit <10-240|unlimited>` and `launcher.cfg` "Values: 10 to 240
  or unlimited (0)" (it said 0 to 240, though 1 to 9 were refused).
- Updated rea-launcher-sdk to v0.2.0. The settings window's look cards,
  selection colour and widget sizes now come from the SDK (`Metrics`),
  with OutRun's values; the look cards gain a focus ring and report their
  name, state and note to screen readers. `or-play`'s tests load it with
  the SDK's `REA_FUNCTIONS_ONLY`/`REA_PLAY_HERE` convention
  (`OR_PLAY_FUNCTIONS_ONLY` is gone). CI also runs the Windows-target
  tests under Wine.
- The HD texture pack download runs on the SDK's download module, WinHTTP
  client and download row (same folder, pin, file names and `.complete`
  marker, so a pack downloaded or half-downloaded by an earlier version is
  found and resumed). The HD Pack tab's row says how far it got; **Download**
  resumes an interrupted download (the button was called "Resume download")
  and a failure stays on screen until the next try. The completion marker
  is now written before the folder is moved into place.
- Settings window accessibility: control borders reach 3:1 contrast
  (`line` `#3d3430` → `#6d6663`); the caption on the photo is 13 pt on a
  solid plate instead of 11 pt straight on the picture; the Picture tab is
  grouped (Screen, Detail, Effects, Linux); Remove asks for a second press
  before deleting the 3.2 GB pack; folder errors say what to choose
  instead. A test keeps the palette and text sizes within the SDK's
  checks.
- Every row in the window shows a one-line note of what it changes (before,
  some help was only on mouse hover, out of reach of a pad). Controls that
  can't be used now are shown greyed out with the reason instead of being
  hidden (impulse triggers under Wine or with rumble off, shuffle without
  the CD switcher, rumble strength with rumble off, the HD pack switch
  without a pack). Vsync is a switch; rumble strength has its own row.
- A value saved in `launcher.cfg` or given as a flag that the window
  doesn't offer (e.g. `fps_limit = 75`, `anisotropy = 2`, `1600x900`) is
  shown as an extra choice instead of no choice being lit.
- `launcher.cfg` messages: on/off settings now say "needs on or off"
  (`true`/`false` are still read), and a bad resolution says "needs
  WIDTHxHEIGHT, original or off". A new `launcher.cfg` is written from the
  option table, with the default of every setting in its `#key =` line.
- An unknown `OR2006C2C.exe` message links to the bug report form, and the
  window offers a "Report this version" button; the log records the EXE's
  SHA-256 on every start. The game EXE is found in any spelling.
- `--no-window` uses the HD pack downloaded earlier when `hd_textures` is
  on without a folder, as the window does.

- Built on the shared [rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk)
  v0.1.0: the runtime copy, saves mirror, INI and `launcher.cfg` editing,
  Win32 helpers, the settings window's kit, `or-play`'s session and
  gamescope handling (`sdk/rea-play.sh`, shipped beside `or-play`), the
  AppImage builder and CI now come from it. Settings, files, commands and
  variables stay the same. Differences a player may notice:
  - a save the game deleted stays deleted instead of coming back one
    launch later;
  - `launcher.cfg`: an unknown setting is reported in `launcher.log`, and
    a value set for the first time fills its commented `#key =` line
    instead of being appended at the end;
  - `launcher.log` starts afresh past 1 MiB;
  - the settings window shows a focus ring for keyboard and pad, and the
    d-pad's left/right step the focused row of choices;
  - `or-play` keeps the settings window's errors in
    `~/.config/or-launcher/settings.log` and also accepts `settings`,
    `play`, `--stop`, `--status` and `--help`.

### Fixed

- A game whose `outrun2006.ini` names a display adapter that isn't
  attached (`DX/ADAPTER = 1` from a two-screen PC, or under or-play,
  where DXVK shows one GPU) stopped with "Direct3D can't be set up". The
  launcher now uses adapter 0 for that start and logs a warning; the
  player's value comes back once that adapter is attached again (recorded
  in the runtime's `.or-game-ini-owned`). A valid adapter is left alone.
- `or-play` sizes gamescope from the look when no resolution is set
  (`look =` in `launcher.cfg`, or `--play --look …` for one run): Steam
  Deck 1280×800, Coast to Coast the screen, Original 640×480. Nothing set
  (a first `--play --game`) or an unreadable resolution is the game's
  640×480, as the launcher draws it; it was 1920×1080 around a smaller
  race.
- Under Wine the game now loads the bundled VC++ runtime (`msvcp140`,
  `vcruntime140`, `vcomp140`) as well as Tweaks' `dinput8` and DXVK's
  `d3d9` from its copy whatever started the launcher: the launcher writes
  Wine's per-program overrides for `OR2006C2C.exe`
  (`HKCU\Software\Wine\AppDefaults\OR2006C2C.exe\DllOverrides`, only
  where no value exists; SDK v0.3.3). Wine ignores the
  `WINEDLLOVERRIDES` the launcher passes to the game, so until now it used
  its own VC++ runtime. The launch plan and snapshots list them.
- A `look =` line in `launcher.cfg` now shows in the settings window: it
  opened on Custom at 640×480 although the game played the look's values,
  and pressing Play saved 640×480 over them (seen with the real game,
  2026-11-02).
- `OutRunLauncher --linux` with nothing else opens the settings window
  again, as in 0.1.0 (with the Linux scaler), instead of starting the
  game. The window opens when no option is given; `--no-window`,
  `--check` and `--download-hd-pack` still skip it.
- `--download-hd-pack` while another launcher is open no longer exits 0
  without downloading anything. With the settings window open it says to
  use the HD Pack tab; while the game runs it says so; either way it exits
  with 5.
- Errors that stop the launcher (game folder not found, an unknown
  `OR2006C2C.exe`, a failed install) show a dialog under Wine and Proton
  again, and are printed to `play.log` too. Since the console output
  change they only reached the log there. `--no-window` runs print them
  without a dialog.
- A runtime folder without its own `outrun2006.ini` gets one again: when
  the game folder's copy already held the launcher's settings, the file
  was never written to the runtime.
- `THIRD_PARTY.md` lists every image in the window, the package and the
  README (the icon, the screenshots and the showcase video were missing);
  the origin of the generated hero art and icon is marked as to be
  confirmed.
- `outrun2006.ini` is no longer rewritten whole on every start: the
  launcher sets only the resolution, anti-aliasing and windowed keys and
  keeps the language, adapter and fog. A new runtime folder starts from
  the game folder's copy. Version 0.1.0 reset the language to English
  (`LANGUAGE = 0`) on every start, so a runtime copy it wrote takes the
  language, adapter and fog from the game folder's copy once, on the
  first start after the update (`.or-game-ini` in the runtime records
  it). A language picked in the game after that stays.
- Saves: the runtime copy records each save file as of the last copy
  (`.or-saves`). At start, a save changed in the saves folder since then
  (a restored backup, saves from another PC) goes into the game; otherwise
  a save the game wrote but never copied back (the session ended before
  the write-back) goes to the saves folder instead of being overwritten.
  Without a record the saves folder wins, as before.
- Settings window under Wine: created at its final size on large screens
  instead of resized after creation, which XWayland with fractional
  scaling can lose (the window stayed small or clipped). Wine's own DPI
  setting (e.g. 144 or 192) is taken into account, so the window is not
  enlarged twice.
- `or-play stop` ends the game's session (and Wine in our prefix) instead
  of or-play's own shell; the lock holds the session's pid and start time,
  so a stale lock or a reused pid is never signalled and another
  OR2006C2C.exe no longer blocks a launch.
- `or-play`'s generated script takes the launcher path as an argument
  instead of pasting it into its source (a path with a quote broke it).
- A failure to write Wine's controller settings (winebus) is logged
  instead of ignored.
- Wine DLL overrides: the player's own `WINEDLLOVERRIDES` rules for the
  DLLs the game needs (`dinput8`, `d3d9`, `mscoree`, `mshtml`) are
  dropped instead of competing with ours; their other rules are kept.
  Wine is also recognised from `WINEPREFIX`, `WINELOADER` or
  `STEAM_COMPAT_DATA_PATH` when a Wine build hides `wine_get_version`.
- `OutRun2006Tweaks.ini`: lines starting with `;` are comments too, and
  keys match without regard to case, so a key typed in another case is
  replaced in place instead of added a second time. Files written by
  earlier versions come out the same.
- The runtime folder is refused inside the game folder (or around it) also
  when it doesn't exist yet, is a relative path, or is reached through a
  link.
- `launcher.cfg` and the runtime copy's manifest are written to a
  temporary file and renamed over the old one, so an interrupted write
  can't leave them half-written.
- An unreadable `launcher.cfg` (no permission) stops the launcher with a
  message instead of being taken for a first run and written over; a file
  saved as ANSI is read (non-UTF-8 characters replaced) instead of being
  ignored.
- The runtime-folder overlap check also works on drives Windows can't
  canonicalize (some RAM disks, `subst` drives, network shares): those
  paths are compared as text.
- Saving settings on Windows retries for half a second when an antivirus
  or OneDrive briefly holds `launcher.cfg`.
- `or-play` interrupted (Ctrl+C, or its terminal closed) also stops Wine
  in our prefix and waits for the game's session to end; a game started
  by or-play 0.1.0 (pid-only lock) still counts as running.
- Wine DLL overrides: a player's rule spelled with `.dll` (`D3D9.DLL=b`)
  is recognised as a rule for that DLL too. The README says the
  launcher's rules replace the player's own for its DLLs (a `d3d9` rule
  to leave DXVK is not applied).

### Changed

- `or-play` gives gamescope the race size as its output size too
  (`-W/-H`), and keeps SDL reading pads in the background
  (`SDL_*_ALLOW_BACKGROUND_EVENTS=1` unless already set).
- The settings window opens on the recommended look only when there is no
  `launcher.cfg` yet (or it is empty), as in the sibling launchers; a
  saved file that matches the template word for word is kept as the
  player's.
- AppImage: built with a pinned, dated AppImage runtime (`20251108`,
  SHA-256 checked) instead of whatever appimagetool downloads, and carries
  the README, licences, third-party list and Linux guide under
  `usr/share/doc/or`.
- Releases carry a `SHA256SUMS` file and take their notes from the
  version's CHANGELOG section (`packaging/release-notes.sh`); CI builds
  with `--locked`, fails when an artifact is missing, and checks the
  shell scripts' syntax.

## [0.1.0] - 2026-11-02

The first release: a launcher that plays your own copy of OutRun 2006:
Coast 2 Coast on Windows 10/11 and on Linux (Proton, Wine, AppImage).

### Added

- Settings window (Play, Picture, HD Pack, Controls, Extras) with the
  Coast to Coast, Original 2006 and Steam Deck looks, usable with a
  gamepad, and the same settings in a commented `launcher.cfg` and on the
  command line.
- A private runtime copy of the game, synced from the player's folder,
  which is never changed; saves kept in a folder of the player's choice.
- OutRun2006Tweaks (CI build `08e5efb4`) bundled from a release of this
  repository, its replacement EXE and `dinput8.dll` installed into the
  runtime copy, `OutRun2006Tweaks.ini` and `outrun2006.ini` written from
  the settings on every start; keys the in-game overlay changed are reset.
- The x86 Visual C++ runtime deployed app-local; DXVK and the DLL overrides
  under Wine and Proton; XInput as the input API under Wine and Proton.
- The "Rush a Difficulty" fix (Tweaks' xdelta patch applied to the runtime
  copy), reversible.
- HD Pack tab: downloads the community HD texture pack on request (resume,
  SHA-256 check, free-space check, progress), unpacks it and points the
  game at it in place; `--download-hd-pack` does the same without the
  window.
- `linux/or-play`: the settings window, then the game in gamescope at the
  chosen size and scaler, on the best GPU.
- Linux AppImage with a portable Wine 11.20.
- Packaging from pinned, checksummed downloads; CI that tests, lints,
  builds and publishes the Windows zip and the AppImage; a weekly workflow
  proposing newer OutRun2006Tweaks builds.

[Unreleased]: https://github.com/akitaonrails/rea-outrun-2006/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/akitaonrails/rea-outrun-2006/releases/tag/v0.1.0
