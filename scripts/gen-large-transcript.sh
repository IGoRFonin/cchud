#!/usr/bin/env bash
# Phase 6 Task 9 — generate ~50 MB transcript fixture for hyperfine.
# Output is git-ignored (see .gitignore). Idempotent: skips if file exists
# and has expected size.

set -euo pipefail
cd "$(dirname "$0")/.."

OUT="benches/samples/transcripts/large-50mb.jsonl"
PAYLOAD_OUT="benches/samples/payload-with-transcript-large.json"
TARGET_BYTES=$((50 * 1024 * 1024))

mkdir -p "$(dirname "$OUT")"

if [[ -f "$OUT" ]]; then
  current=$(wc -c <"$OUT" | tr -d ' ')
  if [[ "$current" -ge "$TARGET_BYTES" ]]; then
    echo "$OUT already $current bytes — skipping"
    exit 0
  fi
fi

> "$OUT"
i=0
while [[ "$(wc -c <"$OUT" | tr -d ' ')" -lt "$TARGET_BYTES" ]]; do
  ts_user=$(printf '2026-01-01T%02d:%02d:00Z' $(( (i / 60) % 24 )) $(( i % 60 )))
  ts_assistant=$(printf '2026-01-01T%02d:%02d:05Z' $(( (i / 60) % 24 )) $(( i % 60 )))
  in_tokens=$(( 100 + (i % 500) ))
  out_tokens=$(( 50 + (i % 200) ))
  cache_r=$(( 1000 + (i % 5000) ))
  cache_c=$(( i % 100 ))
  printf '{"type":"user","timestamp":"%s"}\n' "$ts_user" >> "$OUT"
  printf '{"type":"assistant","timestamp":"%s","message":{"usage":{"input_tokens":%d,"output_tokens":%d,"cache_read_input_tokens":%d,"cache_creation_input_tokens":%d}},"thinking":{"effort":"high"}}\n' \
    "$ts_assistant" "$in_tokens" "$out_tokens" "$cache_r" "$cache_c" >> "$OUT"
  i=$((i + 1))
done

cat > "$PAYLOAD_OUT" <<EOF
{
  "session_id": "bench-large",
  "model": {"id": "claude-sonnet-4-6", "display_name": "Sonnet 4.6"},
  "workspace": {"current_dir": "/tmp"},
  "transcript_path": "$(pwd)/$OUT"
}
EOF

echo "Generated $OUT ($(wc -c <"$OUT" | tr -d ' ') bytes)"
echo "Payload: $PAYLOAD_OUT"
