#!/usr/bin/env bash
set -euo pipefail

# Run web dev servers and open a page.
# Usage:
#   bash run_web.sh                       # opens REPL (platforms/web/repl)
#   bash run_web.sh repl                  # same as default
#   bash run_web.sh visual                # opens a minimal visual page (REPL server)
#   bash run_web.sh healthcheck           # opens the headless healthcheck index page (Node COOP/COEP)
#   bash run_web.sh gallery               # opens a visual gallery that runs all JS examples
#   PORT=9000 bash run_web.sh healthcheck # custom port for Node server (healthcheck/gallery)
#   REPL_PORT=5190 bash run_web.sh repl   # custom Vite dev server port

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PAGE="${1:-repl}"

# Utilities
is_port_free() {
  local p="$1"
  if command -v lsof >/dev/null 2>&1; then
    ! lsof -nP -iTCP:"$p" -sTCP:LISTEN >/dev/null 2>&1
  else
    # Fallback: assume free (Vite/Node will fail if not)
    return 0
  fi
}

pick_free_port() {
  local base="$1"
  local max_incr=200
  local i=0
  local candidate="$base"
  while [ "$i" -le "$max_incr" ]; do
    if is_port_free "$candidate"; then
      echo "$candidate"
      return 0
    fi
    i=$((i+1))
    candidate=$((base+i))
  done
  # Last resort
  echo $(( (RANDOM % 10000) + 20000 ))
}

# Ensure local WASM pkg is built and distributed to subprojects
"$ROOT_DIR/build_web.sh" --debug

if [ "$PAGE" = "healthcheck" ] || [ "$PAGE" = "gallery" ]; then
  # Use Node COOP/COEP static server for the healthcheck harness
  if ! command -v node >/dev/null 2>&1; then
    echo "Node.js is required. Install it and re-run." >&2
    exit 1
  fi

  PORT="${PORT:-}"
  if [ -z "${PORT}" ]; then
    PORT="$(pick_free_port 8765)"
  fi
  echo "[run_web] healthcheck server port: $PORT"

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

  # Wait for server (probe an existing page to avoid 404 on "/")
  ATTEMPTS=0
  PROBE_PATH="/healthcheck/index.html"
  if [ "$PAGE" = "gallery" ]; then
    PROBE_PATH="/healthcheck/gallery.html"
  fi
  until curl -sSf "http://localhost:$PORT$PROBE_PATH" >/dev/null 2>&1; do
    ATTEMPTS=$((ATTEMPTS+1))
    if [ "$ATTEMPTS" -gt 80 ]; then
      echo "Server did not start in time" >&2
      exit 1
    fi
    sleep 0.25
  done

  if [ "$PAGE" = "gallery" ]; then
    URL="http://localhost:$PORT/healthcheck/gallery.html"
  else
    # Default to skipping texture-target examples for now; override with ?skipTexture=0 if needed
    URL="http://localhost:$PORT/healthcheck/index.html?skipTexture=1"
  fi

  # Open the browser
  if command -v open >/dev/null 2>&1; then
    open "$URL"
  elif command -v xdg-open >/dev/null 2>&1; then
    xdg-open "$URL" >/dev/null 2>&1 || true
  else
    echo "Open this URL in your browser: $URL"
  fi

  # Keep logs interactive
  wait "$SERVER_PID"
  exit 0
fi

# REPL / Visual via Vite from bindings folder (repl)
if ! command -v pnpm >/dev/null 2>&1; then
  echo "pnpm is required. Install it (e.g., brew install pnpm) and re-run." >&2
  exit 1
fi

REPL_DIR="$ROOT_DIR/platforms/web/repl"
REPL_PORT="${REPL_PORT:-}"
if [ -z "${REPL_PORT}" ]; then
  REPL_PORT="$(pick_free_port 5173)"
fi

echo "[run_web] repl vite port: $REPL_PORT"

# Fresh install to ensure vite plugins are present
rm -rf "$REPL_DIR/node_modules"
pnpm install --dir "$REPL_DIR"

# Start Vite dev server on the chosen port and open the requested page
pnpm --dir "$REPL_DIR" dev -- --port "$REPL_PORT" --strictPort &
SERVER_PID=$!
cleanup() {
  if ps -p "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" || true
  fi
}
trap cleanup EXIT

# Wait for Vite
ATTEMPTS=0
until curl -sSf "http://localhost:$REPL_PORT" >/dev/null 2>&1; do
  ATTEMPTS=$((ATTEMPTS+1))
  if [ "$ATTEMPTS" -gt 240 ]; then
    echo "Dev server did not start in time" >&2
    exit 1
  fi
  sleep 0.25
done

if [ "$PAGE" = "visual" ]; then
  URL="http://localhost:$REPL_PORT/visual.html"
else
  # Default to the REPL index
  URL="http://localhost:$REPL_PORT/"
fi

if command -v open >/dev/null 2>&1; then
  open "$URL"
elif command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$URL" >/dev/null 2>&1 || true
else
  echo "Open this URL in your browser: $URL"
fi

wait "$SERVER_PID"
