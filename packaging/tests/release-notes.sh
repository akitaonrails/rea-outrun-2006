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
check() { # name expected actual
  if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$3'"; fails=$((fails + 1)); fi
}
check "the tag's section only" "$(printf '### Fixed\n\n- two')" "$(sh "$NOTES" v0.2.0 "$T/CHANGELOG.md")"
check "without the v" "$(printf 'The first release.\n\n- one')" "$(sh "$NOTES" 0.1.0 "$T/CHANGELOG.md")"
if sh "$NOTES" v0.9.0 "$T/CHANGELOG.md" > /dev/null 2>&1; then check "a missing version fails" fail ok; else echo "ok   a missing version fails"; fi
if sh "$NOTES" v0.3.0 "$T/empty.md" > /dev/null 2>&1; then check "an empty section fails" fail ok; else echo "ok   an empty section fails"; fi
sh "$NOTES" v0.1.0 "$HERE/../../CHANGELOG.md" > /dev/null && echo "ok   this repository's 0.1.0 notes" || { echo "FAIL repository notes"; fails=$((fails + 1)); }
[ "$fails" = 0 ] && echo "all passed" || { echo "$fails failed"; exit 1; }
