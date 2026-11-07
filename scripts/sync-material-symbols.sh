#!/usr/bin/env bash
# Keeps reference/material-symbols/ an exact copy of the Material Symbols this library ships (ADR 0017): every icon
# at 24 px, in the outlined, rounded and sharp styles, filled and not, from google/material-design-icons at the pinned
# commit. The pin is the commit in the `material-symbols/` row of reference/README.md.
#
#   scripts/sync-material-symbols.sh pinned         print the pinned commit
#   scripts/sync-material-symbols.sh check          fail if the copy differs from the pinned commit
#   scripts/sync-material-symbols.sh update REV     copy the files at REV and pin it
#
# After an update, run `cargo xtask gen-icons` to regenerate the Rust icons.
set -euo pipefail

repo_url="https://github.com/google/material-design-icons"
copy="reference/material-symbols"
styles=(outlined rounded sharp)

root="$(git rev-parse --show-toplevel)"
cd "$root"

pinned_rev() {
    sed -n 's/^| `material-symbols\/` | .* | `\([0-9a-f]\{40\}\)` | .*$/\1/p' reference/README.md
}

current="$(pinned_rev)"
if [[ ! "$current" =~ ^[0-9a-f]{40}$ ]]; then
    echo "error: no commit found in the material-symbols row of reference/README.md" >&2
    exit 1
fi

mode="${1:-}"
case "$mode" in
    pinned)
        echo "$current"
        exit 0
        ;;
    check) rev="$current" ;;
    update) rev="${2:-}" ;;
    *)
        echo "usage: $0 pinned | check | update REV" >&2
        exit 2
        ;;
esac
if [[ ! "$rev" =~ ^[0-9a-f]{40}$ ]]; then
    echo "error: not a full commit hash: $rev" >&2
    exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
# The repository holds every icon in 168 variants per style and size, about 2 million files: fetch one commit without
# its trees or blobs, then check out only the license and the 24 px files of the default grade and weight, filled and
# not (24,942 files, about 13 MB, half a minute). The variants of another grade or weight are named `_grad…` and
# `_wght…`, which no icon name contains.
git -C "$work" init -q
git -C "$work" fetch -q --depth 1 --filter=tree:0 "$repo_url" "$rev"
git -C "$work" sparse-checkout set --no-cone /LICENSE \
    '/symbols/web/*/materialsymbols*/*_24px.svg' \
    '!/symbols/web/*/materialsymbols*/*_grad*_24px.svg' \
    '!/symbols/web/*/materialsymbols*/*_wght*_24px.svg'
git -C "$work" checkout -q FETCH_HEAD

# Every icon has both files in every style; one missing is an error, not an icon left out.
expected="$work/expected"
for style in "${styles[@]}"; do
    mkdir -p "$expected/$style"
done
cp "$work/LICENSE" "$expected/LICENSE-APACHE"
for dir in "$work"/symbols/web/*/; do
    name="$(basename "$dir")"
    for style in "${styles[@]}"; do
        for file in "${name}_24px.svg" "${name}_fill1_24px.svg"; do
            source="$dir/materialsymbols$style/$file"
            if [[ ! -f "$source" ]]; then
                echo "error: material-design-icons $rev has no symbols/web/$name/materialsymbols$style/$file" >&2
                exit 1
            fi
            cp "$source" "$expected/$style/$file"
        done
    done
done

matches() {
    diff -r "$expected" "$copy" >/dev/null 2>&1
}

if [[ "$mode" == "check" ]]; then
    if ! matches; then
        diff -rq "$expected" "$copy" 2>&1 | sed "s|$expected|material-design-icons $rev|" | head -20 >&2 || true
        echo "error: $copy differs from material-design-icons $rev; run scripts/sync-material-symbols.sh update $rev" >&2
        exit 1
    fi
    echo "Material Symbols match the pinned commit $rev"
    exit 0
fi

if matches; then
    echo "Material Symbols unchanged at $rev; still pinned to $current"
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

echo "Material Symbols copied from $rev (was $current); now run: cargo xtask gen-icons"
