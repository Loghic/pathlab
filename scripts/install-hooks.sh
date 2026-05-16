#!/usr/bin/env bash
#
# Install the pre-commit hook for this repo.
#
# Usage:  ./scripts/install-hooks.sh
#
# This symlinks scripts/pre-commit into .git/hooks/pre-commit. Using a
# symlink (rather than a copy) means future updates to the script are
# picked up automatically with no re-install.

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
hook_src="$repo_root/scripts/pre-commit"
hook_dst="$repo_root/.git/hooks/pre-commit"

if [ ! -f "$hook_src" ]; then
    printf 'Source hook not found: %s\n' "$hook_src" >&2
    exit 1
fi

# Refuse to clobber an existing real file silently.
if [ -e "$hook_dst" ] && [ ! -L "$hook_dst" ]; then
    printf 'A non-symlink %s already exists.\n' "$hook_dst" >&2
    printf 'Move it aside (e.g. mv %s %s.bak) and re-run.\n' "$hook_dst" "$hook_dst" >&2
    exit 1
fi

ln -sf "$hook_src" "$hook_dst"
chmod +x "$hook_src"

printf 'Pre-commit hook installed at %s\n' "$hook_dst"
printf 'It runs:  cargo fmt --check  ->  cargo clippy -D warnings  ->  cargo test\n'
printf 'Skip a single commit with:  git commit --no-verify\n'
