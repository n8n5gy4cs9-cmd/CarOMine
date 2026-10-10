#!/bin/sh
# usage: tools/commit.sh T-xxx "message"  -> commit all + push to main
set -e
cd "$(dirname "$0")/.."
git add -A
git commit -q -m "$1: $2

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TpX39jkmWCvrsyuuyEUzNe"
git push -q origin main
