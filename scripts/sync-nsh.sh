#!/usr/bin/env bash
# Keeps the vendored NSH skills on the NSH commit pinned in the root Cargo.toml (ADR 0006).
#
#   scripts/sync-nsh.sh check          fail if .claude/skills differs from the pinned commit's skills
#   scripts/sync-nsh.sh update [REV]   pin NSH to REV (default: the newest commit of NSH's main),
#                                      copy its skills and refresh Cargo.lock
set -euo pipefail

repo_url="https://github.com/longbridge/nsh"
skills=(nsh nsh-design-guides)

root="$(git rev-parse --show-toplevel)"
cd "$root"

pinned_rev() {
    sed -n 's/^nsh-core = { git = "https:\/\/github\.com\/longbridge\/nsh", rev = "\([0-9a-f]\{40\}\)", version = "[^"]*" }$/\1/p' Cargo.toml
}

# The NSH version Cargo.toml declares beside the rev (ADR 0009).
pinned_version() {
    sed -n 's/^nsh-core = { git = "[^"]*", rev = "[0-9a-f]\{40\}", version = "\([^"]*\)" }$/\1/p' Cargo.toml
}

# NSH's own version at the fetched commit: its workspace package version.
nsh_version() {
    sed -n '/^\[workspace\.package\]$/,/^\[/s/^version = "\([^"]*\)"$/\1/p' "$work/Cargo.toml" | head -n 1
}

current="$(pinned_rev)"
if [[ ! "$current" =~ ^[0-9a-f]{40}$ ]]; then
    echo "error: no NSH rev found for nsh-core in Cargo.toml" >&2
    exit 1
fi
# Every NSH crate (nsh-core, nsh-base, and the test-only nsh-engine and nsh-testing) is pinned to the same rev.
nsh_deps="$(grep -c '^nsh-[a-z-]* = { git = "https://github.com/longbridge/nsh"' Cargo.toml)"
if [[ "$nsh_deps" -lt 2 || "$(grep -c "rev = \"$current\"" Cargo.toml)" -ne "$nsh_deps" ]]; then
    echo "error: every NSH crate in Cargo.toml must share the rev of nsh-core" >&2
    exit 1
fi

mode="${1:-}"
case "$mode" in
    check) rev="$current" ;;
    update) rev="${2:-$(git ls-remote "$repo_url" refs/heads/main | cut -f1)}" ;;
    *)
        echo "usage: $0 check | update [REV]" >&2
        exit 2
        ;;
esac
if [[ ! "$rev" =~ ^[0-9a-f]{40}$ ]]; then
    echo "error: not a full commit hash: $rev" >&2
    exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
git -C "$work" init -q
git -C "$work" fetch -q --depth 1 "$repo_url" "$rev"
git -C "$work" checkout -q FETCH_HEAD

expected="$work/expected"
mkdir -p "$expected"
for skill in "${skills[@]}"; do
    cp -R "$work/skills/$skill" "$expected/$skill"
    cp "$work/LICENSE-APACHE" "$expected/$skill/LICENSE-APACHE"
done

version="$(nsh_version)"
if [[ -z "$version" ]]; then
    echo "error: no workspace version in NSH's Cargo.toml at $rev" >&2
    exit 1
fi

if [[ "$mode" == "check" ]]; then
    status=0
    if [[ "$(pinned_version)" != "$version" ]]; then
        echo "error: Cargo.toml declares NSH $(pinned_version), but NSH $rev is $version; run scripts/sync-nsh.sh update $rev" >&2
        status=1
    fi
    for skill in "${skills[@]}"; do
        if ! diff -r "$expected/$skill" ".claude/skills/$skill" >/dev/null; then
            echo "error: .claude/skills/$skill differs from NSH $rev; run scripts/sync-nsh.sh update $rev" >&2
            status=1
        fi
    done
    [[ "$status" -eq 0 ]] && echo "NSH skills match the pinned commit $rev"
    exit "$status"
fi

for skill in "${skills[@]}"; do
    rm -rf ".claude/skills/$skill"
    cp -R "$expected/$skill" ".claude/skills/$skill"
done

if [[ "$rev" != "$current" ]]; then
    # Every file that names the pinned commit moves to the new one; vendored copies are left alone.
    git grep -l "$current" -- . ':!.claude/skills/nsh' ':!.claude/skills/nsh-design-guides' ':!reference' |
        while IFS= read -r file; do
            sed -i.bak "s/$current/$rev/g" "$file" && rm -f "$file.bak"
        done
fi
old_version="$(pinned_version)"
if [[ "$version" != "$old_version" ]]; then
    sed -i.bak "s/rev = \"$rev\", version = \"$old_version\"/rev = \"$rev\", version = \"$version\"/" Cargo.toml && rm -f Cargo.toml.bak
fi
if [[ "$rev" != "$current" || "$version" != "$old_version" ]]; then
    # Resolves the new rev into Cargo.lock without updating anything else.
    cargo metadata --format-version 1 >/dev/null
fi

echo "NSH pinned to $rev (was $current)"
