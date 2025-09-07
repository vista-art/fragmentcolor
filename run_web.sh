#!/usr/bin/env bash
set -euo pipefail

# Run dev servers from the repo root and open a page.
# Usage:
#   bash run_web.sh                  # opens JS example index (single-shader circle)
#   bash run_web.sh multipass        # opens JS example multipass page
#   bash run_web.sh headless         # opens JS example headless page
#   bash run_web.sh repl             # opens local REPL (platforms/web/repl) using local WASM pkg

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PAGE="${1:-repl}"

if ! command -v pnpm >/dev/null 2>&1; then
  echo "pnpm is required. Install it (e.g., brew install pnpm) and re-run." >&2
  exit 1
fi

if [ "$PAGE" = "repl" ]; then
  EX_DIR="$ROOT_DIR/platforms/web/repl"
  # Ensure local WASM pkg is built for the REPL to import
  "$ROOT_DIR/build_web.sh" --debug
else
  EX_DIR="$ROOT_DIR/examples/javascript"
fi

# Install dependencies if needed
rm -rf "$EX_DIR/node_modules"
pnpm install --dir "$EX_DIR"

# Start dev server
pnpm --dir "$EX_DIR" dev &
SERVER_PID=$!

cleanup() {
  if ps -p "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" || true
  fi
}
trap cleanup EXIT

# Wait for Vite to be ready
ATTEMPTS=0
until curl -sSf "http://localhost:5173" >/dev/null 2>&1; do
  ATTEMPTS=$((ATTEMPTS+1))
  if [ "$ATTEMPTS" -gt 120 ]; then
    echo "Dev server did not start in time" >&2
    exit 1
  fi
  sleep 0.5
done

URL="http://localhost:5173/"
if [ "$PAGE" = "multipass" ]; then
  URL="http://localhost:5173/multipass.html"
fi

# Open the browser
if command -v open >/dev/null 2>&1; then
  open "$URL"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$URL" >/dev/null 2>&1 || true
else
  echo "Open this URL in your browser: $URL"
fi

# Keep server in foreground until user stops it
wait "$SERVER_PID"
