#!/bin/sh
# usage: tools/commit.sh T-xxx "message"  -> cargo check, commit all, push to main.  NOCHECK=1 to skip the build gate.
set -e
cd "$(dirname "$0")/.."
if [ -z "$NOCHECK" ] && [ -f Cargo.toml ]; then cargo check -q 2>&1 | grep -E "^error" -A6 && { echo "BUILD BROKEN - not committing"; exit 1; } || true; fi
git add -A
git commit -q -m "$1: $2

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TpX39jkmWCvrsyuuyEUzNe"
git push -q origin main
