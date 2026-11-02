#!/bin/sh
# Updates the bundled OutRun2006Tweaks build (see docs/TWEAKS.md).
#
#   packaging/update-tweaks.sh [--run <id>] [--publish] [--force]
#
# Upstream publishes current builds only as GitHub Actions artifacts, which
# expire after 90 days. This script takes one (the newest successful master
# build, or --run <id>), repacks just what the launcher ships plus the
# licence and a SOURCE.txt into a reproducible zip, and compares its
# binaries with the build packaging/tweaks.txt pins now.
#
# With --publish it creates the release tweaks-<commit> on $OR_REPO
# (default akitaonrails/rea-outrun-2006) holding that zip and the upstream
# source of the same commit, and points packaging/tweaks.txt at it.
# Without, it only reports (a dry run); the zip is left in .work/tweaks.
# --force publishes even when the binaries match the pinned build (to move
# a pin onto this repository's releases).
#
# Needs gh (authenticated), curl, unzip, zip and sha256sum. Prints
# "changed=yes|no" last, for CI.
set -eu
ROOT=$(dirname "$(dirname "$(readlink -f "$0")")")
UPSTREAM=emoose/OutRun2006Tweaks
REPO=${OR_REPO:-akitaonrails/rea-outrun-2006}
FILES="dinput8.dll OR2006C2C.exe OutRun2006Tweaks.ini OutRun2006Tweaks.lods.ini"
CACHE=${OR_CACHE:-$HOME/.cache/or-package}
PINS=$ROOT/packaging
. "$PINS/pins.sh"

RUN= PUBLISH= FORCE=
while [ $# -gt 0 ]; do
  case $1 in
    --run) RUN=$2; shift 2 ;;
    --publish) PUBLISH=1; shift ;;
    --force) FORCE=1; shift ;;
    *) echo "usage: $0 [--run <id>] [--publish] [--force]"; exit 2 ;;
  esac
done

# The build: the newest successful master run that still has its artifact.
if [ -z "$RUN" ]; then
  for id in $(gh api "repos/$UPSTREAM/actions/runs?branch=master&status=success&per_page=20" \
      --jq '.workflow_runs[].id'); do
    if gh api "repos/$UPSTREAM/actions/runs/$id/artifacts" --jq '.artifacts[] | select(.expired | not) | .name' | grep -q .; then
      RUN=$id; break
    fi
  done
  [ -n "$RUN" ] || { echo "no upstream build with a live artifact"; exit 1; }
fi
SHA=$(gh api "repos/$UPSTREAM/actions/runs/$RUN" --jq .head_sha)
SHORT=$(printf %.8s "$SHA")
DATE=$(gh api "repos/$UPSTREAM/commits/$SHA" --jq .commit.committer.date)
ARTIFACT=$(gh api "repos/$UPSTREAM/actions/runs/$RUN/artifacts" \
  --jq '[.artifacts[] | select(.expired | not)][0].name')
[ -n "$ARTIFACT" ] || { echo "run $RUN has no live artifact"; exit 1; }
echo "upstream: run $RUN, commit $SHORT ($DATE), artifact $ARTIFACT"

WORK=$ROOT/.work/tweaks/$SHORT
rm -rf "$WORK"; mkdir -p "$WORK/raw" "$WORK/zip"
# nightly.link serves public artifacts without a login.
curl -fsSL "https://nightly.link/$UPSTREAM/actions/runs/$RUN/$ARTIFACT.zip" -o "$WORK/artifact.zip"
unzip -q "$WORK/artifact.zip" -d "$WORK/raw"
for f in $FILES; do
  [ -f "$WORK/raw/$f" ] || { echo "the artifact lacks $f; the layout changed, check by hand"; exit 1; }
  cp "$WORK/raw/$f" "$WORK/zip/$f"
done
curl -fsSL "https://raw.githubusercontent.com/$UPSTREAM/$SHA/LICENSE.md" -o "$WORK/zip/LICENSE.md"
LICENCE=$(head -n 3 "$WORK/zip/LICENSE.md" | tr -s ' \n' ' ' | cut -c1-60)
cat > "$WORK/zip/SOURCE.txt" <<EOF
OutRun2006Tweaks by emoose, built by its GitHub Actions workflow.

Commit:   https://github.com/$UPSTREAM/commit/$SHA ($DATE)
Build:    https://github.com/$UPSTREAM/actions/runs/$RUN
Artifact: $ARTIFACT.zip, sha256 $(sha256sum "$WORK/artifact.zip" | cut -d' ' -f1)
Source:   https://github.com/$UPSTREAM/archive/$SHA.tar.gz
Licence:  LICENSE.md at that commit ($LICENCE...)

Repacked for rea-outrun-2006 with the files the launcher uses; the
binaries are byte-identical to the artifact's.
EOF
# Reproducible: fixed timestamps and order, no extra attributes.
find "$WORK/zip" -type f -exec touch -d "$DATE" {} +
ZIP=$WORK/OutRun2006Tweaks-$SHORT.zip
(cd "$WORK/zip" && TZ=UTC zip -q -X -D "$ZIP" $FILES LICENSE.md SOURCE.txt)
DIGEST=$(sha256sum "$ZIP" | cut -d' ' -f1)

# Same binaries as the pinned build? Then there is nothing to do.
CURRENT=$(fetch tweaks)
mkdir -p "$WORK/current"
unzip -q -o "$CURRENT" -d "$WORK/current"
same=1
for f in $FILES; do
  if ! cmp -s "$WORK/zip/$f" "$WORK/current/$f"; then
    same=; echo "changed: $f"
  fi
done
if [ -n "$same" ] && [ -z "$FORCE" ]; then
  echo "the pinned build already has these binaries"
  echo "changed=no"
  exit 0
fi
echo "new build: $ZIP"
echo "  sha256 $DIGEST"
echo "  OR2006C2C.exe sha256 $(sha256sum "$WORK/zip/OR2006C2C.exe" | cut -d' ' -f1)"

if [ -z "$PUBLISH" ]; then
  echo "dry run: rerun with --publish to release it and update packaging/tweaks.txt"
  echo "changed=yes"
  exit 0
fi

TAG=tweaks-$SHORT
curl -fsSL "https://github.com/$UPSTREAM/archive/$SHA.tar.gz" -o "$WORK/OutRun2006Tweaks-$SHORT-source.tar.gz"
gh release create "$TAG" --repo "$REPO" --title "OutRun2006Tweaks $SHORT" \
  --notes-file "$WORK/zip/SOURCE.txt" --latest=false \
  "$ZIP" "$WORK/OutRun2006Tweaks-$SHORT-source.tar.gz"
URL=https://github.com/$REPO/releases/download/$TAG/OutRun2006Tweaks-$SHORT.zip
cat > "$ROOT/packaging/tweaks.txt" <<EOF
# OutRun2006Tweaks (emoose), the engine of the fixes: tweaks/ gets
# dinput8.dll, OR2006C2C.exe, the two INI files, LICENSE.md and SOURCE.txt.
# Upstream ships current builds only as expiring CI artifacts, so each
# build we use is repacked and kept as a release of this repository by
# packaging/update-tweaks.sh (docs/TWEAKS.md); never edit this by hand.
# Commit $SHORT ($DATE), build run $RUN.
# One line: URL SHA-256.
$URL $DIGEST
EOF
echo "published $TAG; packaging/tweaks.txt updated"
echo "changed=yes"
