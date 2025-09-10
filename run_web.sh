#!/usr/bin/env bash
set -euo pipefail

# Run a local COOP/COEP Node static server from platforms/web and open a page.
# Usage:
#   bash run_web.sh                  # opens visual healthcheck page
#   bash run_web.sh visual           # same as default
#   bash run_web.sh healthcheck      # opens the headless healthcheck index page
#   PORT=9000 bash run_web.sh        # custom port

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PORT="${PORT:-8765}"
PAGE="${1:-visual}"

# Ensure WASM pkg exists for local dev
"$ROOT_DIR/build_web.sh" --debug

if ! command -v node >/dev/null 2>&1; then
  echo "Node.js is required. Install it and re-run." >&2
  exit 1
fi

# Start Node COOP/COEP server
ART_DIR="$ROOT_DIR/platforms/web/healthcheck/playwright-artifacts"
mkdir -p "$ART_DIR" >/dev/null 2>&1 || true
(
  cd "$ROOT_DIR" && PORT="$PORT" node platforms/web/healthcheck/serve.mjs
) &
SERVER_PID=$!

cleanup() {
  if ps -p "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" || true
  fi
}
trap cleanup EXIT

# Wait for server
ATTEMPTS=0
until curl -sSf "http://localhost:$PORT" >/dev/null 2>&1; do
  ATTEMPTS=$((ATTEMPTS+1))
  if [ "$ATTEMPTS" -gt 40 ]; then
    echo "Server did not start in time" >&2
    exit 1
  fi
  sleep 0.25
done

if [ "$PAGE" = "healthcheck" ]; then
  URL="http://localhost:$PORT/healthcheck/index.html"
else
  URL="http://localhost:$PORT/healthcheck/visual.html"
fi

# Open the browser
if command -v open >/dev/null 2>&1; then
  open "$URL"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$URL" >/dev/null 2>&1 || true
else
  echo "Open this URL in your browser: $URL"
fi

# Keep server running until user stops it
wait "$SERVER_PID"
