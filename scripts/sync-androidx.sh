#!/usr/bin/env bash
# Keeps reference/androidx-material3-tokens/ an exact copy of the androidx token files at the pinned commit
# (ADR 0003). The pin is the commit in the `androidx-material3-tokens/` row of reference/README.md.
#
#   scripts/sync-androidx.sh pinned         print the pinned commit
#   scripts/sync-androidx.sh check          fail if the copy differs from the pinned commit
#   scripts/sync-androidx.sh update [REV]   copy the token files at REV (default: the newest commit of androidx-main)
#                                           and pin it, but only when the files change: androidx-main moves many
#                                           times a day, the tokens rarely
#
# After an update, run `cargo xtask gen-tokens` to regenerate the Rust tokens.
set -euo pipefail

repo_url="https://github.com/androidx/androidx"
branch="androidx-main"
tokens_path="compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens"
copy="reference/androidx-material3-tokens"

root="$(git rev-parse --show-toplevel)"
cd "$root"

pinned_rev() {
    sed -n 's/^| `androidx-material3-tokens\/` | .* | `\([0-9a-f]\{40\}\)` | .*$/\1/p' reference/README.md
}

current="$(pinned_rev)"
if [[ ! "$current" =~ ^[0-9a-f]{40}$ ]]; then
    echo "error: no androidx commit found in the androidx-material3-tokens row of reference/README.md" >&2
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
# androidx is a monorepo of several gigabytes: fetch one commit without its trees or blobs, then check out only the
# license and the tokens folder (about 13 MB, a few seconds).
git -C "$work" init -q
git -C "$work" fetch -q --depth 1 --filter=tree:0 "$repo_url" "$rev"
git -C "$work" sparse-checkout set --no-cone /LICENSE.txt "/$tokens_path/"
git -C "$work" checkout -q FETCH_HEAD

expected="$work/expected"
mkdir -p "$expected"
cp "$work/$tokens_path"/*.kt "$expected/"
cp "$work/LICENSE.txt" "$expected/LICENSE-APACHE"

if [[ "$mode" == "check" ]]; then
    if ! diff -r "$expected" "$copy" >/dev/null; then
        diff -rq "$expected" "$copy" | sed "s|$expected|androidx $rev|" >&2 || true
        echo "error: $copy differs from androidx $rev; run scripts/sync-androidx.sh update $rev" >&2
        exit 1
    fi
    echo "androidx tokens match the pinned commit $rev"
    exit 0
fi

if diff -r "$expected" "$copy" >/dev/null; then
    echo "androidx tokens unchanged at $rev; still pinned to $current"
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

echo "androidx tokens copied from $rev (was $current); now run: cargo xtask gen-tokens"
