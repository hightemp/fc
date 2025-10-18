#!/usr/bin/env bash
set -Eeuo pipefail

# gen_bench.sh — generate many files and benchmark fc vs ls|wc
# Usage: ./scripts/gen_bench.sh [COUNT=1000000] [DIR=./testdir]
# Example: ./scripts/gen_bench.sh 200000 ./testdir

COUNT="${1:-1000000}"
DIR="${2:-./testdir}"

if ! [[ "$COUNT" =~ ^[0-9]+$ ]]; then
  echo "COUNT must be a non-negative integer" >&2
  exit 1
fi

NPROC="$(getconf _NPROCESSORS_ONLN 2>/dev/null || nproc 2>/dev/null || echo 1)"

echo "[1/4] Preparing directory: $DIR (files: $COUNT)"
rm -rf "$DIR"
mkdir -p "$DIR"

echo "[2/4] Generating files..."
# Parallel create empty files: file_1 ... file_$COUNT
seq 1 "$COUNT" | xargs -n1 -P"$NPROC" -I{} touch "$DIR/file_{}"

echo "[3/4] Building fc (release)..."
cargo build --release >/dev/null

FC_BIN="target/release/fc"
if [[ ! -x "$FC_BIN" ]]; then
  echo "fc binary not found at $FC_BIN" >&2
  exit 1
fi

# Quick sanity check
FC_COUNT="$("$FC_BIN" "$DIR")"
LS_COUNT="$(LC_ALL=C ls -1U "$DIR" | wc -l)"
echo "Sanity: fc=$FC_COUNT, ls|wc=$LS_COUNT"

run_time() {
  local label="$1"; shift
  echo "[4/4] Benchmark: $label"
  if command -v /usr/bin/time >/dev/null 2>&1; then
    /usr/bin/time -f "real=%E user=%U sys=%S maxrss=%MKB" "$@"
  else
    time "$@"
  fi
}

# Benchmark fc (suppress count output, keep timing and stderr)
run_time "fc" "$FC_BIN" "$DIR" >/dev/null

# Benchmark ls | wc -l (unsorted, C locale)
run_time "ls -1U | wc -l" bash -c "LC_ALL=C ls -1U \"$DIR\" | wc -l" >/dev/null

echo "Done."