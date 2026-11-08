#!/usr/bin/env bash
# Keeps reference/material-web/ an exact copy of the one Material Web file that holds the elevation formula this library
# draws shadows with (ADR 0018): `elevation/internal/_elevation.scss` of material-components/material-web at the pinned
# commit, with the repository's license. The pin is the commit in the `material-web/` row of reference/README.md.
#
#   scripts/sync-material-web.sh pinned         print the pinned commit
#   scripts/sync-material-web.sh check          fail if the copy differs from the pinned commit
#   scripts/sync-material-web.sh update [REV]   copy the file at REV (default: the newest commit of main) and pin it,
#                                               but only when the file changes: main moves often, this file rarely
#
# After an update, check `crates/nsh-m3e/src/elevation.rs` against the new file: its tests read the vendored file and
# fail when the formula they transcribe is no longer in it.
set -euo pipefail

repo_url="https://github.com/material-components/material-web"
branch="main"
source_path="elevation/internal/_elevation.scss"
copy="reference/material-web"

root="$(git rev-parse --show-toplevel)"
cd "$root"

pinned_rev() {
    sed -n 's/^| `material-web\/` | .* | `\([0-9a-f]\{40\}\)` | .*$/\1/p' reference/README.md
}

current="$(pinned_rev)"
if [[ ! "$current" =~ ^[0-9a-f]{40}$ ]]; then
    echo "error: no commit found in the material-web row of reference/README.md" >&2
    exit 1
fi

mode="${1:-}"
case "$mode" in
    pinned)
        echo "$current"
        exit 0
        ;;
    check) rev="$current" ;;
    update) rev="${2:-$(git ls-remote "$repo_url" "refs/heads/$branch" | cut -f1)}" ;;
    *)
        echo "usage: $0 pinned | check | update [REV]" >&2
        exit 2
        ;;
esac
if [[ ! "$rev" =~ ^[0-9a-f]{40}$ ]]; then
    echo "error: not a full commit hash: $rev" >&2
    exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
# Material Web has many components and their tests: fetch one commit without its trees or blobs, then check out only
# the license and the one file (a few kilobytes, a second or two).
git -C "$work" init -q
git -C "$work" fetch -q --depth 1 --filter=tree:0 "$repo_url" "$rev"
git -C "$work" sparse-checkout set --no-cone /LICENSE "/$source_path"
git -C "$work" checkout -q FETCH_HEAD

# A file missing at the revision is an error, not an empty copy.
if [[ ! -f "$work/$source_path" ]]; then
    echo "error: material-web $rev has no $source_path" >&2
    exit 1
fi
expected="$work/expected"
mkdir -p "$expected"
cp "$work/$source_path" "$expected/"
cp "$work/LICENSE" "$expected/LICENSE-APACHE"

# The copy matches what Material Web has at the revision, including a copy that is missing.
matches() {
    diff -r "$expected" "$copy" >/dev/null 2>&1
}

if [[ "$mode" == "check" ]]; then
    if ! matches; then
        diff -rq "$expected" "$copy" 2>&1 | sed "s|$expected|material-web $rev|" | head -20 >&2 || true
        echo "error: $copy differs from material-web $rev; run scripts/sync-material-web.sh update $rev" >&2
        exit 1
    fi
    echo "Material Web's elevation matches the pinned commit $rev"
    exit 0
fi

if matches; then
    echo "Material Web's elevation unchanged at $rev; still pinned to $current"
    exit 0
fi

rm -rf "$copy"
cp -R "$expected" "$copy"

if [[ "$rev" != "$current" ]]; then
    # Every file that names the pinned commit moves to the new one; the vendored copy is left alone.
    git grep -l "$current" -- . ":!$copy" |
        while IFS= read -r file; do
            sed -i.bak "s/$current/$rev/g" "$file" && rm -f "$file.bak"
        done
fi

echo "Material Web's elevation copied from $rev (was $current); now check crates/nsh-m3e/src/elevation.rs against it"
