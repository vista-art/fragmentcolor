#!/usr/bin/env bash
set -euo pipefail

# Run the JavaScript examples dev server from the repo root and open a page.
# Usage:
#   bash ren_web.sh           # opens index (single-shader circle)
#   bash ren_web.sh multipass # opens multipass example
#   bash ren_web.sh headless  # opens headless example

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EX_DIR="$ROOT_DIR/examples/javascript"
PAGE="${1:-index}"

if ! command -v pnpm >/dev/null 2>&1; then
  echo "pnpm is required. Install it (e.g., brew install pnpm) and re-run." >&2
  exit 1
fi

# Install dependencies if needed
if [ ! -d "$EX_DIR/node_modules" ]; then
  pnpm install --dir "$EX_DIR"
fi

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
