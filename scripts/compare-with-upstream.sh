#!/usr/bin/env bash
# scripts/compare-with-upstream.sh
# Manual visual diff against upstream ccstatusline. Requires ccstatusline
# in PATH. Logs to plan/phase-4/manual-test-log.md.

set -euo pipefail

if ! command -v ccstatusline >/dev/null 2>&1; then
    echo "ccstatusline not in PATH — skipping manual comparison."
    exit 0
fi

CONFIGS=(
    benches/configs/phase-4-23w-powerline.json
)

for cfg in "${CONFIGS[@]}"; do
    for payload in benches/samples/payload-cchud-*.json; do
        upstream=$(cat "$payload" | ccstatusline 2>/dev/null || echo "")
        ours=$(cat "$payload" | CCHUD_CONFIG="$cfg" CCHUD_TEST_COLOR_LEVEL=true-color target/release/cchud)
        if [[ "$upstream" != "$ours" ]]; then
            echo "DIFF in $cfg / $payload"
        else
            echo "OK $cfg / $payload"
        fi
    done
done
