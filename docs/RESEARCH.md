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
