#!/usr/bin/env bash
set -euo pipefail

APP_PROCESS_PATTERN="${WARP_PROFILE_PROCESS_PATTERN:-^/Applications/WarpCodexOss\\.app/Contents/MacOS/warp-oss$}"
LOG_FILE="${WARP_PROFILE_LOG_FILE:-$HOME/Library/Logs/warp-codex-oss.log}"
OUT_DIR="${WARP_PROFILE_OUT_DIR:-/tmp/warpcodexoss-perf-$(date +%Y%m%d-%H%M%S)}"
SAMPLE_SECONDS="${WARP_PROFILE_SAMPLE_SECONDS:-10}"

PID="${WARP_PROFILE_PID:-}"
if [[ -z "$PID" ]]; then
  PID="$(pgrep -f "$APP_PROCESS_PATTERN" | head -n 1 || true)"
fi

if [[ -z "$PID" ]]; then
  echo "WarpCodexOss process not found."
  echo "Expected process pattern: $APP_PROCESS_PATTERN"
  echo "Start WarpCodexOss, then run this script again."
  exit 1
fi

mkdir -p "$OUT_DIR"

{
  echo "WarpCodexOss performance capture"
  echo "captured_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "pid=$PID"
  echo "sample_seconds=$SAMPLE_SECONDS"
  echo "out_dir=$OUT_DIR"
  echo "process_pattern=$APP_PROCESS_PATTERN"
  echo "markdown_perf_env=${WARP_MARKDOWN_PERF_LOG:-<unset>}"
} | tee "$OUT_DIR/summary.txt"

PS_FIELDS="pid,ppid,stat,%cpu,%mem,rss,vsz,etime,command"
ps -o "$PS_FIELDS" -p "$PID" > "$OUT_DIR/process.txt"
CHILD_PIDS="$(pgrep -P "$PID" || true)"
if [[ -n "$CHILD_PIDS" ]]; then
  ps -o "$PS_FIELDS" -p "$(echo "$CHILD_PIDS" | paste -sd, -)" > "$OUT_DIR/child-processes.txt" || true
else
  echo "No child processes." > "$OUT_DIR/child-processes.txt"
fi
top -l 2 -pid "$PID" -stats pid,cpu,mem,threads,ports,time,command > "$OUT_DIR/top.txt" || true
sample "$PID" "$SAMPLE_SECONDS" -file "$OUT_DIR/sample.txt" || true
vmmap -summary "$PID" > "$OUT_DIR/vmmap-summary.txt" || true
lsof -p "$PID" > "$OUT_DIR/open-files.txt" || true

if [[ -f "$LOG_FILE" ]]; then
  tail -n 500 "$LOG_FILE" > "$OUT_DIR/warp-log-tail.txt"
  grep -E "warpcodexoss\\.markdown_perf|stage=(markdown_|file_notebook_|svg_)" "$LOG_FILE" > "$OUT_DIR/markdown-perf.log" || true
else
  echo "Log file not found: $LOG_FILE" > "$OUT_DIR/warp-log-tail.txt"
  : > "$OUT_DIR/markdown-perf.log"
fi

cat > "$OUT_DIR/README.txt" <<EOF
How to use this capture:

1. Open WarpCodexOss with WARP_MARKDOWN_PERF_LOG=1 when you want Markdown/SVG timing logs.
2. Open a large Markdown file, scroll, zoom, and wait until rendering settles.
3. Run script/dev/profile_warpcodexoss.sh from the repo.
4. Inspect:
   - summary.txt: capture metadata
   - process.txt/top.txt: live CPU and memory snapshot
   - sample.txt: CPU stack sample
   - vmmap-summary.txt: memory breakdown
   - markdown-perf.log: Markdown parse, preview reset, zoom, SVG parse/raster timing
   - warp-log-tail.txt: recent app warnings/errors
EOF

echo "Capture written to $OUT_DIR"
