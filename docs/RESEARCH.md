# Community fixes this launcher builds on

What the community already made for OutRun 2006: Coast 2 Coast (PC, 2006),
what each piece fixes, its licence and upkeep, and what the launcher does
with it. Research first, then build only what nobody has solved (the
SDK's `docs/STANDARDS.md`, "Research before building"). Sources:
PCGamingWiki's
[OutRun 2006: Coast 2 Coast](https://www.pcgamingwiki.com/wiki/OutRun_2006:_Coast_2_Coast)
page, the projects' own repositories, ProtonDB/WineHQ for Linux.

| Project | What it fixes | Licence | Maintained | Decision |
| --- | --- | --- | --- | --- |
| [OutRun2006Tweaks](https://github.com/emoose/OutRun2006Tweaks) (emoose) | Nearly everything: a DRM-free replacement EXE, widescreen and UI scaling, frame-rate cap and interpolation, an SDL input system (triggers, rumble, impulse triggers, hot-plug), draw distance and LOD fixes, reflections, anisotropy, the Xbox brightness and sky glow, the unused hi-def characters, texture replacement (the HD pack), the CD switcher, bug fixes, the community online server | MIT up to commit `021e4bf5`, GPL-3.0 from it on | yes (CI builds; tagged releases are old) | **Bundled** (build `08e5efb4`, repacked as a release of this repository, `docs/TWEAKS.md`); its INI is generated from `launcher.cfg`. The DRM-free EXE is used by owner decision (2026-11-02, `AGENTS.md`). |
| OutRun2006Tweaks Soundtrack Fix 1.0 | "Rush a Difficulty" misses its first two seconds on PC: an xdelta3 patch of the player's own song | none of its own (published with Tweaks) | no (one-off) | **Bundled** with the official xdelta3 3.1.0 (GPL-2.0); applied to the launcher's copy of the song only |
| [OR2006Sprites HD pack](https://github.com/envido32/OR2006Sprites) (Envido32's compilation of [Sonic-TV](https://github.com/Sonic-TV/OR2006Sprites) and contributors) | Redrawn menus, HUD, sprites and button prompts at high resolution | none stated | yes | **Download on request** (HD Pack tab, `--download-hd-pack`; pinned, `docs/HD_TEXTURES.md`); never bundled or mirrored |
| [DXVK](https://github.com/doitsujin/dxvk) | Direct3D 9 through Vulkan under Wine/Proton | zlib | yes | **Bundled** (32-bit `d3d9.dll`, Wine only); mirrored |
| Microsoft VC++ 2015–2022 x86 runtime | Needed by the Tweaks DLL; most PCs have only the x64 one | Visual Studio Distributable Code | yes | **Bundled** app-local (three DLLs); the installer is mirrored |
| SpecialK `d3d9.dll` (shipped by some repacks) | General D3D tweaks | GPL-3.0 | yes | **Not used**: it loads before the real D3D9 and fights Tweaks/DXVK; the runtime copy never takes the game folder's `d3d9.dll` |
| Community online server (clarissa.port0.org) | Online play after SEGA's servers closed | — | yes | **Configured** through Tweaks' `DemonwareServerOverride` (Tweaks' default) |

## Known `OR2006C2C.exe` builds

The launcher identifies the game by its EXE's SHA-256
(`crates/or-launcher/src/edition.rs`) and always plays its own bundled
Tweaks EXE in its private copy. A build is added only when we can verify
where it comes from. An EXE that isn't in this table is accepted when the
game's data matches the 2006 release (next section).

| SHA-256 | Size | What it is | Verified from |
| --- | --- | --- | --- |
| `bdafa88a5abdd2a9743f6bdcc5e2189288c0203790412fce10b9bb649933482f` | 14,950,400 | the 2006 EXE (DRM-wrapped, PE timestamp 2006-05-26) | the owner's install |
| `68ceb386829066f8455b9d027320af962584321f3e2e8a79c72841495a6134c3` | 3,674,112 | OutRun2006Tweaks' DRM-free EXE, Large Address Aware | every upstream release from v0.3.0-rc1 to v0.6.1, and the owner's install |
| `c926e54a5620e8e47199caf532235c25ddaba512da198bde15db1ee4375f806f` | 3,674,112 | the same Tweaks EXE without the Large Address Aware flag (one byte) | upstream releases v0.1 to v0.2.5 |
| `4876712f501640951d892069db826e431aede3e4ab87686fc1d556e26df9fec1` | 962,560 | the 2007-2010 Steam release (app 4730, delisted 2010), packed | **reported, not checked by us**: the same digest and size from two Steam owners who bought it before the delisting (issue #1) |

The Steam row is the one build we accept without a copy of our own,
because the launcher never runs the game's EXE: it identifies the folder
by it, then plays its bundled Tweaks EXE in its private copy. What the
Steam EXE is comes from OutRun2006Tweaks (2026-11-09): its README says the
replacement EXE "should be compatible with both the Steam release & the
original DVD version", and `src/hooks_bugfixes.cpp` (`FixC2CRankings`)
notes that for the Steam release "they repacked the original
pre-securom-wrapper 2006 game EXE without any changes". No Tweaks issue
mentions this size or digest, and no store page or wiki publishes it.
Nothing else the launcher does depends on the EXE; the soundtrack fix
patches the song only when it has the known truncated size. The Steam
release's data files have not been played by us.

## Identifying the game by its data (v0.3.7, issues #3 and #4)

DVD owners kept being refused as unknown editions: the 2006 EXE is
SecuROM-wrapped, and its digest differs between regions and pressings
(#4's European DVD ISO, `OutRun-2006-Coast-2-Coast_Win_EN-FR-DE-IT-ES_ISO`,
gave a digest other than our repack's `bdafa88a…`). The EXE digest is the
wrong question for this launcher:

- The game's `OR2006C2C.exe` is never run, patched or copied: the runtime
  copy's policy owns `or2006c2c.exe` (`runtime.rs`), and the launcher
  places its bundled Tweaks EXE there (`tweaks.rs`, `plan.rs`). Tweaks'
  EXE is the 2006 game code without the wrapper, so what has to be right
  is the data it loads.
- No other step depends on the EXE. The soundtrack fix patches "Rush a
  Difficulty" only when the song has the known truncated size
  (3,055,933 bytes) and leaves any other file alone; the INI writers, the
  saves mirror, DXVK and the HD pack don't look at the edition. The
  edition's `patched` flag is informational.

So when the EXE is unknown, the launcher checks five data files by size
and SHA-256 (`DATA_2006`) and plays when all match, saying in the window
and `launcher.log` that the EXE wasn't recognised and isn't used. When
one differs, it refuses as before, naming the EXE's SHA-256 and the first
file that differs.

| File | Size | SHA-256 (in full in `edition.rs`) |
| --- | --- | --- |
| `Common/obj_course_obj_common_pmt.sz` | 844,991 | `0cfc9a4d…` |
| `Common/def_orhiscores.bin` | 67,680 | `426ad7d8…` |
| `Driver/obj_driver_rival_pmt.sz` | 1,138,698 | `132618f4…` |
| `Stage/PALM_BT/CS_CS_PALM_BT_bin.sz` | 56,073 | `93d84b3f…` |
| `Stage/YOSE_R/CS_CS_YOSE_R_bin.sz` | 19,811 | `e0768983…` |

The files were chosen to be language-free (the European DVD carries five
languages in `Text/`, other regions may not), outside the folders mods
replace (the owner's FXT repack changed files in `Sprite/`, `RVT/` and
`effects/`; players replace songs in `Sound/`, and the soundtrack fix
rewrites one), and two of them are Coast 2 Coast stages a demo would
lack. They are identical in the owner's install and in its repack
installer (extracted separately with innoextract, the installer that
holds the `bdafa88a…` EXE). Both copies come from one European release,
so another region whose data differs is still refused, with the file
named; that case needs a player's report. Hashing the five files takes
about 60 ms.

## Owner decisions

- **2026-11-02: OutRun2006Tweaks' DRM-free replacement `OR2006C2C.exe` is
  approved** (recorded in `AGENTS.md` and `THIRD_PARTY.md` that day). The
  2006 EXE is wrapped in disc DRM that fails on modern Windows; the
  launcher drops the maintained mod's replacement EXE into its private
  copy verbatim and never writes its own crack or patches the game's EXE.
  Waived for `launcher-check` in `.rea-check-waivers`.

## What we built ourselves

Nothing that patches the game: the launcher only arranges and configures
the projects above. Our own findings so far are Wine/Proton settings:

- Under Wine/Proton, Tweaks' default `InputBackend = 0`
  (Windows.Gaming.Input) leaves the d-pad dead in menus; XInput (3) works.
  The launcher defaults to XInput under Wine (owner play-test,
  2026-11-02). Drafted for PCGamingWiki and upstream in
  `docs/CONTRIBUTIONS.md`.
- GNU ld 2.47 drops one of two `kernel32.dll` import descriptors in our
  own launcher build; we link with lld (`docs/RUNTIME_NOTES.md`). Not a
  game issue.

## Old-game trouble spots (SDK checklist)

| Spot | Status |
| --- | --- |
| Timers and frame pacing on fast CPUs | handled by Tweaks (fixed 60 Hz logic, its own limiter) |
| Multi-core affinity | no known problem |
| DirectSound3D/EAX audio | no known problem; not checked under Wine beyond normal play |
| CD audio | none (OGG files on disk) |
| Intro videos | none needing a codec; Tweaks can skip the logos |
| Resolution and aspect tables | handled by Tweaks (any size, widescreen UI) |
| Losing focus | Tweaks' borderless window; SDL background events exported by `or-play` (pending check, `docs/RUNTIME_NOTES.md`) |
