# OutRun2006Tweaks: how it is bundled and updated

[OutRun2006Tweaks](https://github.com/emoose/OutRun2006Tweaks) by emoose is
the engine of the fixes. The launcher ships one build of it in `tweaks/`
next to `OutRunLauncher.exe` and installs it into its private copy of the
game on every start:

| File | What it is |
| --- | --- |
| `dinput8.dll` | the mod itself, loaded by the game as DirectInput |
| `OR2006C2C.exe` | the mod's DRM-free replacement for the 2006 EXE |
| `OutRun2006Tweaks.ini` | the mod's settings with all upstream comments; the launcher rewrites only the keys it owns |
| `OutRun2006Tweaks.lods.ini` | the mod's list of objects kept at full detail with a longer draw distance |
| `LICENSE.md`, `SOURCE.txt` | the build's licence and where it came from |

## Where the build comes from

Upstream's tagged releases are old (v0.6.1, 2024). Current builds exist
only as GitHub Actions artifacts, which GitHub deletes after 90 days. So
each build the launcher uses is **repacked and kept as a release of this
repository**, named `tweaks-<commit>`:

- `OutRun2006Tweaks-<commit>.zip`: the five files above plus `SOURCE.txt`.
  The binaries are byte-identical to upstream's artifact; the zip has
  fixed timestamps, so the same build always repacks to the same bytes.
- `OutRun2006Tweaks-<commit>-source.tar.gz`: upstream's source at that
  commit (the project is GPL-3.0 from commit `021e4bf5` on; MIT before).

`packaging/tweaks.txt` pins that zip by URL and SHA-256.
`packaging/package.sh` fetches it, checks the digest and unpacks it into
`tweaks/`. Nothing else in the package build talks to upstream.

The launcher recognises a game folder whose `OR2006C2C.exe` is the 2006
original, a known Tweaks replacement EXE, or the replacement EXE in its
own `tweaks/` folder, so a folder patched with the build we ship is
always accepted.

## Updating

**Automatic.** The `tweaks-update` workflow runs every Monday (and on
demand from the Actions tab, optionally with an upstream run id). It runs
`packaging/update-tweaks.sh`, which takes the newest successful upstream
build of `master` and compares its binaries with the pinned build. When
they differ it publishes the `tweaks-<commit>` release and opens a pull
request `chore(tweaks): update OutRun2006Tweaks to <commit>` that changes
only `packaging/tweaks.txt`. When they match, it does nothing.

Pull requests opened by a workflow don't start other workflows: close and
reopen the pull request (or push to its branch) to run the build.

**By hand**, with an authenticated `gh`:

```sh
packaging/update-tweaks.sh                    # dry run: newest build, report only
packaging/update-tweaks.sh --run <id>         # a specific upstream Actions run
packaging/update-tweaks.sh --publish          # release it and repin
```

The repacked zip is left in `.work/tweaks/<commit>/` for inspection.
`--force` publishes even when the binaries match the pinned build (used to
move the pin from nightly.link onto this repository).

## Before merging an update

1. **Play-test it.** Build the package from the pull request's branch
   (`packaging/package.sh`), play a race, and note what was checked in
   `docs/RUNTIME_NOTES.md`.
2. **Check the licence.** `LICENSE.md` in the release zip; update the
   OutRun2006Tweaks row of `THIRD_PARTY.md` if it changed.
3. **Check the INI.** Compare the new `OutRun2006Tweaks.ini` with the
   previous one. If a key the launcher writes (see `apply_tweaks` in
   `crates/or-launcher/src/config.rs`) was renamed or moved to another
   section, update the launcher; the unit tests check each key's section.
4. Merge. The next build and release use the new pin.

To go back to an earlier build, point `packaging/tweaks.txt` at its
`tweaks-<commit>` release again. Releases are never replaced or deleted:
an update for a build that already has one pins the zip that's there.
