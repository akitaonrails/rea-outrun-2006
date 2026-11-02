# Contributions back to the community

Fixes found while building this launcher, drafted for the places players
look. The owner reviews and posts them; nothing here is posted by an
agent. Only findings verified in the real game are drafted as ready.

| Id | Finding | Target | Status |
| --- | --- | --- | --- |
| O1 | Under Wine/Proton, OutRun2006Tweaks' `InputBackend = 0` (Windows.Gaming.Input) loses the d-pad in menus; `InputBackend = 3` (XInput) works | PCGamingWiki *OutRun 2006*, Linux note; an issue at emoose/OutRun2006Tweaks | **ready** (owner play-test, 2026-11-02); not posted |

Evidence: `docs/RUNTIME_NOTES.md`, "2026-11-02 — the owner's play-test
before v0.1.0" (Wine 11.21, gamescope, Xbox-style pad, Tweaks build
`08e5efb4`).

## O1 — PCGamingWiki: d-pad in menus under Wine/Proton

Page: <https://www.pcgamingwiki.com/wiki/OutRun_2006:_Coast_2_Coast>. Add
to the existing "Linux/SteamOS" area, or as an `{{ii}}` under Input:

```wikitext
{{ii}} With [[#OutRun2006Tweaks|OutRun2006Tweaks]]' new input system under Wine/Proton, the default <code>InputBackend = 0</code> (Windows.Gaming.Input) leaves the D-pad dead in menus (only the stick moves the selection); set <code>InputBackend = 3</code> (XInput) in {{file|OutRun2006Tweaks.ini}}.<ref>{{Refcheck|user=OWNER|date=2026-11-02|comment=Wine 11.21, gamescope, Xbox-style pad, Tweaks build 08e5efb4.}}</ref>
```

## O1 upstream — emoose/OutRun2006Tweaks issue

Search existing issues first. #54, #116, #199, #141, #319 and #296 are
related but not this.

> **Title:** Under Wine/Proton, InputBackend = 0 (WGI) ignores the D-pad in menus
>
> Build: master `08e5efb4` (CI run 32972770753), `UseNewInput = true`.
> Wine 11.21 (also expected on Proton), Xbox-style XInput pad.
>
> With `InputBackend = 0` (Windows.Gaming.Input) the D-pad does nothing in
> the menus, and the left stick moves the selection only sometimes. Driving
> works. With `InputBackend = 3` (XInput) the D-pad works in menus.
> `ControllerHotPlug` was off.
>
> Wine's Windows.Gaming.Input is incomplete, so this is likely Wine's side.
> A note in the INI comment or the README's Linux line ("under Wine/Proton
> use InputBackend = 3") would save Linux players the search. Happy to test
> builds.

## Not ready (needs a real-game check first)

- `SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS=1` /
  `SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1` for OutRun under Wine: exported
  by `or-play`, not yet verified to matter.
