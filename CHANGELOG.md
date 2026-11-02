# Changelog

All notable changes to this project. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- Saves: the runtime copy records each save file as of the last copy
  (`.or-saves`). At start, a save changed in the saves folder since then
  (a restored backup, saves from another PC) goes into the game; otherwise
  a save the game wrote but never copied back (the session ended before
  the write-back) goes to the saves folder instead of being overwritten.
  Without a record the saves folder wins, as before.
- Settings window under Wine: created at its final size on large screens
  instead of resized after creation, which XWayland with fractional
  scaling can lose (the window stayed small or clipped).
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
