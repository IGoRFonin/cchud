#!/usr/bin/env bash
set -euo pipefail

if [ ! -x "./target/release/cchud" ]; then
  echo "ERROR: ./target/release/cchud not found. Run 'cargo build --release' first." >&2
  exit 1
fi

# Берём первые 3 семпла (отсортированно для детерминизма)
SAMPLES=$(ls benches/samples/*.json 2>/dev/null | sort | head -3)

if [ -z "$SAMPLES" ]; then
  echo "ERROR: no JSON samples in benches/samples/" >&2
  exit 1
fi

for SAMPLE in $SAMPLES; do
  echo "### $(basename "$SAMPLE")"
  echo
  hyperfine \
    --warmup 20 \
    --runs 200 \
    --shell=none \
    --input "$SAMPLE" \
    "./target/release/cchud" \
    --export-markdown -
  echo
done

# --- Phase 2: cchud vs ccstatusline comparison ---

if command -v ccstatusline >/dev/null 2>&1; then
  COMPARISON_SAMPLE="benches/samples/payload-cchud-sonnet-xlarge.json"
  echo "### Phase 2 comparison: cchud vs ccstatusline"
  echo
  echo "Sample: $(basename "$COMPARISON_SAMPLE")"
  echo
  hyperfine \
    --warmup 20 \
    --runs 200 \
    --shell=none \
    --input "$COMPARISON_SAMPLE" \
    --command-name "cchud" \
    "./target/release/cchud" \
    --command-name "ccstatusline" \
    "ccstatusline" \
    --export-markdown -
  echo
else
  echo "### Phase 2 comparison: SKIPPED (ccstatusline not in PATH)"
fi
