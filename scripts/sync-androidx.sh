#!/usr/bin/env bash
# Keeps reference/androidx-material3-tokens/ an exact copy of the androidx token files at the pinned commit
# (ADR 0003), and reference/androidx-material3-components/ of the androidx components this library follows where they
# override a token (ADR 0012). The pin is the commit in the `androidx-material3-tokens/` row of reference/README.md.
#
#   scripts/sync-androidx.sh pinned         print the pinned commit
#   scripts/sync-androidx.sh check          fail if the copy differs from the pinned commit
#   scripts/sync-androidx.sh update [REV]   copy the files at REV (default: the newest commit of androidx-main) and
#                                           pin it, but only when the files change: androidx-main moves many times a
#                                           day, these files rarely
#
# After an update, run `cargo xtask gen-tokens` to regenerate the Rust tokens.
set -euo pipefail

repo_url="https://github.com/androidx/androidx"
branch="androidx-main"
tokens_path="compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens"
copy="reference/androidx-material3-tokens"
components_path="compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3"
components_copy="reference/androidx-material3-components"
# The component files a component of this library follows (ADR 0012), each named for the androidx file.
components=(Button.kt IconButton.kt IconButtonDefaults.kt)

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
# license, the tokens folder and the component files (about 14 MB, a few seconds).
patterns=(/LICENSE.txt "/$tokens_path/")
for file in "${components[@]}"; do
    patterns+=("/$components_path/$file")
done
git -C "$work" init -q
git -C "$work" fetch -q --depth 1 --filter=tree:0 "$repo_url" "$rev"
git -C "$work" sparse-checkout set --no-cone "${patterns[@]}"
git -C "$work" checkout -q FETCH_HEAD

expected="$work/expected"
mkdir -p "$expected"
cp "$work/$tokens_path"/*.kt "$expected/"
cp "$work/LICENSE.txt" "$expected/LICENSE-APACHE"
expected_components="$work/expected-components"
mkdir -p "$expected_components"
for file in "${components[@]}"; do
    cp "$work/$components_path/$file" "$expected_components/"
done
cp "$work/LICENSE.txt" "$expected_components/LICENSE-APACHE"

# Both copies match what androidx has at the revision, including a copy that is missing.
matches() {
    diff -r "$expected" "$copy" >/dev/null 2>&1 && diff -r "$expected_components" "$components_copy" >/dev/null 2>&1
}

if [[ "$mode" == "check" ]]; then
    if ! matches; then
        {
            diff -rq "$expected" "$copy" 2>&1 | sed "s|$expected|androidx $rev|"
            diff -rq "$expected_components" "$components_copy" 2>&1 | sed "s|$expected_components|androidx $rev|"
        } >&2 || true
        echo "error: $copy or $components_copy differs from androidx $rev; run scripts/sync-androidx.sh update $rev" >&2
        exit 1
    fi
    echo "androidx tokens and components match the pinned commit $rev"
    exit 0
fi

if matches; then
    echo "androidx tokens and components unchanged at $rev; still pinned to $current"
    exit 0
fi

rm -rf "$copy" "$components_copy"
cp -R "$expected" "$copy"
cp -R "$expected_components" "$components_copy"

if [[ "$rev" != "$current" ]]; then
    # Every file that names the pinned commit moves to the new one; the vendored copies are left alone.
    git grep -l "$current" -- . ":!$copy" ":!$components_copy" |
        while IFS= read -r file; do
            sed -i.bak "s/$current/$rev/g" "$file" && rm -f "$file.bak"
        done
fi

echo "androidx tokens and components copied from $rev (was $current); now run: cargo xtask gen-tokens"
