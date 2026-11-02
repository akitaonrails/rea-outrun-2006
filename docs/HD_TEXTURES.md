# The HD texture pack

The community HD texture pack redraws the game's menus, HUD and 2D sprites
(tachometers, flags, button prompts for Xbox, PlayStation, Switch and
keyboard) at high resolution. It is
[Envido32's compilation](https://github.com/envido32/OR2006Sprites) of the
[Sonic-TV OR2006Sprites](https://github.com/Sonic-TV/OR2006Sprites) work by
many contributors, and it is the pack the OutRun2006Tweaks README points
to.

It is **a separate download**, not part of the launcher: 326 MB zipped,
3.2 GB unpacked, and published without a licence, so this project links to
it instead of redistributing it.

## Using it

1. Download the newest `OR2-HD-GUI-…zip` from
   <https://github.com/envido32/OR2006Sprites/releases>. The launcher's
   **Get it…** button (Picture tab, after ticking *HD texture pack*) opens
   that page.
2. Unzip it anywhere you like, for example `D:\Games\OutRun HD pack`, or
   `~/Games/OutRun HD pack` on Linux. The zip holds a `textures\load`
   folder.
3. In the launcher's Picture tab, tick **HD texture pack** and choose the
   folder you unzipped into with **Browse…**. The note under it turns gold
   ("Pack found") when the folder is right. The folder above `textures`,
   `textures` itself or `load` all work.
4. Press Play.

The game reads the pack where you unzipped it; nothing is copied into the
launcher's copy of the game. Unticking the option keeps the folder for next
time.

With the pack on, the launcher also restores Clarissa's Japanese
appearance (`RestoreJPClarissa`), which the pack's sprites are drawn for.

In `launcher.cfg` the same settings are:

```ini
hd_textures = on
hd_textures_folder = D:\Games\OutRun HD pack
```

Under Wine or Proton, give the folder as Wine sees it, e.g.
`Z:\home\you\Games\OutRun HD pack` (the Browse button does this for you).
Without `hd_textures_folder`, a `textures-pack` folder next to
`OutRunLauncher.exe` is used.

If the pack is on but its folder is missing or wrong, the game starts with
its own textures and `launcher.log` says why.
