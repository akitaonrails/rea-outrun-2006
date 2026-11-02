#!/bin/sh
# Tests packaging/release-notes.sh: sh packaging/tests/release-notes.sh
set -u
HERE=$(dirname "$(readlink -f "$0")")
NOTES=$HERE/../release-notes.sh
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
cat > "$T/CHANGELOG.md" <<'MD'
# Changelog

## [Unreleased]

### Fixed

- unreleased fix

## [0.2.0] - 2026-12-01

### Fixed

- two

## [0.1.0] - 2026-11-02

The first release.

- one

[Unreleased]: https://example.com/compare/v0.2.0...HEAD
[0.2.0]: https://example.com/v0.2.0
MD
printf '## [0.3.0] - 2027-01-01\n\n## [0.2.0] - x\n\n- y\n' > "$T/empty.md"
fails=0
# Every awk at hand (AWK picks it in release-notes.sh); mawk and busybox
# awk read bracket expressions differently from gawk.
AWKS=awk
for a in mawk gawk original-awk nawk; do command -v "$a" > /dev/null && AWKS="$AWKS $a"; done
if command -v busybox > /dev/null; then
  printf '#!/bin/sh\nexec busybox awk "$@"\n' > "$T/busybox-awk"; chmod +x "$T/busybox-awk"
  AWKS="$AWKS $T/busybox-awk"
fi
check() { # name expected actual
  if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$3'"; fails=$((fails + 1)); fi
}
for awk in $AWKS; do
  check "$awk: the tag's section only" "$(printf '### Fixed\n\n- two')" "$(AWK=$awk sh "$NOTES" v0.2.0 "$T/CHANGELOG.md")"
  check "$awk: without the v" "$(printf 'The first release.\n\n- one')" "$(AWK=$awk sh "$NOTES" 0.1.0 "$T/CHANGELOG.md")"
  if AWK=$awk sh "$NOTES" v0.9.0 "$T/CHANGELOG.md" > /dev/null 2>&1; then check "$awk: a missing version fails" fail ok; else echo "ok   $awk: a missing version fails"; fi
  if AWK=$awk sh "$NOTES" v0.3.0 "$T/empty.md" > /dev/null 2>&1; then check "$awk: an empty section fails" fail ok; else echo "ok   $awk: an empty section fails"; fi
  AWK=$awk sh "$NOTES" v0.1.0 "$HERE/../../CHANGELOG.md" > /dev/null && echo "ok   $awk: this repository's 0.1.0 notes" || { echo "FAIL repository notes"; fails=$((fails + 1)); }
done
[ "$fails" = 0 ] && echo "all passed" || { echo "$fails failed"; exit 1; }
