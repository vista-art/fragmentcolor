#!/usr/bin/env bash
set -euo pipefail

# Run web dev servers and open a page.
# Usage:
#   bash run_web.sh                  # opens REPL (platforms/web/repl)
#   bash run_web.sh repl             # same as default
#   bash run_web.sh visual           # opens a minimal visual page
#   bash run_web.sh healthcheck      # opens the headless healthcheck index page (Node COOP/COEP)
#   PORT=9000 bash run_web.sh        # custom port for Node server (healthcheck)

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PORT="${PORT:-8765}"
PAGE="${1:-repl}"

# Ensure local WASM pkg is built and distributed to subprojects
"$ROOT_DIR/build_web.sh" --debug

if [ "$PAGE" = "healthcheck" ]; then
  # Use Node COOP/COEP static server for the healthcheck harness
  if ! command -v node >/dev/null 2>&1; then
    echo "Node.js is required. Install it and re-run." >&2
    exit 1
  fi
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

  URL="http://localhost:$PORT/healthcheck/index.html"

  # Open the browser
  if command -v open >/dev/null 2>&1; then
    open "$URL"
  elif command -v xdg-open >/dev/null 2>&1; then
    xdg-open "$URL" >/dev/null 2>&1 || true
  else
    echo "Open this URL in your browser: $URL"
  fi

  wait "$SERVER_PID"
  exit 0
fi

# REPL / Visual via Vite from bindings folder (repl)
if ! command -v pnpm >/dev/null 2>&1; then
  echo "pnpm is required. Install it (e.g., brew install pnpm) and re-run." >&2
  exit 1
fi

REPL_DIR="$ROOT_DIR/platforms/web/repl"
# Fresh install to ensure vite plugins are present
rm -rf "$REPL_DIR/node_modules"
pnpm install --dir "$REPL_DIR"

# Start Vite dev server and open the requested page
pnpm --dir "$REPL_DIR" dev &
SERVER_PID=$!
cleanup() {
  if ps -p "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" || true
  fi
}
trap cleanup EXIT

# Wait for Vite
ATTEMPTS=0
until curl -sSf "http://localhost:5173" >/dev/null 2>&1; do
  ATTEMPTS=$((ATTEMPTS+1))
  if [ "$ATTEMPTS" -gt 120 ]; then
    echo "Dev server did not start in time" >&2
    exit 1
  fi
  sleep 0.5
done

if [ "$PAGE" = "visual" ]; then
  URL="http://localhost:5173/visual.html"
else
  # Default to the REPL index
  URL="http://localhost:5173/"
fi

if command -v open >/dev/null 2>&1; then
  open "$URL"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$URL" >/dev/null 2>&1 || true
else
  echo "Open this URL in your browser: $URL"
fi

wait "$SERVER_PID"
