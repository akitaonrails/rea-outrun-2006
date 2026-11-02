# The HD texture pack

The community HD texture pack redraws the game's menus, HUD and 2D sprites
(tachometers, flags, button prompts for Xbox, PlayStation, Switch and
keyboard) at high resolution. It is
[Envido32's compilation](https://github.com/envido32/OR2006Sprites) of the
[Sonic-TV OR2006Sprites](https://github.com/Sonic-TV/OR2006Sprites) work by
many contributors, and it is the pack the OutRun2006Tweaks README points
to.

It is **optional and never bundled**: 326 MB to download, 3.2 GB unpacked,
and published without a licence. The launcher downloads it from the pack's
own GitHub release when the player asks for it.

## Getting it

Open the launcher's **HD Pack** tab and press **Download**. The launcher:

1. checks there is room (about 3.6 GB free),
2. downloads the pinned release over HTTPS, showing progress, speed and time
   left; **Cancel** stops it and **Download** carries on from where it
   stopped, also after closing the launcher,
3. checks the download's SHA-256 against the pin, and throws a damaged one
   away,
4. unpacks it into `%LOCALAPPDATA%\OutRunLauncher\hd-pack\<release>`
   (inside the Wine prefix on Linux), and marks it complete only when every
   file is out,
5. switches the pack on.

**Play** waits while a download runs. The game reads the pack where it was
unpacked; nothing is copied into the launcher's copy of the game.
**Remove** deletes it again (3.2 GB). With the pack on, the launcher also
restores Clarissa's Japanese appearance (`RestoreJPClarissa`), which the
pack's sprites are drawn for.

Without the window (a Steam Deck in Game Mode, scripts):

```sh
OutRunLauncher.exe --download-hd-pack
```

downloads, unpacks and switches it on, logging progress to `launcher.log`.

## Using a copy you already have

Under **Already have it?** in the HD Pack tab, choose the folder you
unzipped `OR2-HD-GUI-…zip` into (the folder above `textures`, `textures`
itself or `load` all work). The note under it says whether the folder holds
the pack. In `launcher.cfg`:

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

## Moving to a newer pack (maintainers)

`packaging/hd-pack.txt` pins the release: URL, SHA-256, the zip's size and
the unpacked size, on one line. It is compiled into the launcher. To move to
a newer release, download it, then:

```sh
sha256sum OR2-HD-GUI-<version>.zip
stat -c %s OR2-HD-GUI-<version>.zip
unzip -l OR2-HD-GUI-<version>.zip | tail -1   # unpacked bytes
```

Update the line, check the zip still holds `textures/load`, build, and test
a download from the HD Pack tab. Players who had the older pack get the new
one on their next download; the old folder is removed once the new one is
in place.
