#!/usr/bin/env bash
# Run FragmentColor healthchecks (Python and Web) with cargo-like output.
# Usage:
#   ./run_healthchecks              # run all
#   ./run_healthchecks all|*|complete
#   ./run_healthchecks py|python|p  # python only
#   ./run_healthchecks js|javascript|j|web|w|wasm  # web only

set -u -o pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PORT="${PORT:-8765}"

GREEN="\033[1;32m"
RED="\033[1;31m"
RESET="\033[0m"

log_test_ok() {
  local name="$1"
  printf "test %s ... %bOK%b\n" "$name" "$GREEN" "$RESET"
}

log_test_fail() {
  local name="$1"
  printf "test %s ... %bFAILED%b\n" "$name" "$RED" "$RESET"
}

parse_mode() {
  local arg="${1:-all}"
  local arg_lc
  arg_lc=$(printf '%s' "$arg" | tr '[:upper:]' '[:lower:]')
  case "$arg_lc" in
    p|py|python) echo "py" ;;
    j|js|javascript|w|web|wasm) echo "web" ;;
    all|complete|\*) echo "all" ;;
    "") echo "all" ;;
    *) echo "all" ;;
  esac
}

run_py() {
  local name="platforms.python.healthcheck"
  # Enable verbose tracing if requested
  if [ "${FC_HEALTHCHECK_VERBOSE:-0}" = "1" ]; then
    export FRAGMENTCOLOR_TRACE=1
    export RUST_BACKTRACE=${RUST_BACKTRACE:-1}
  fi
  # Build a fresh wheel to ensure latest sources are tested
  if bash "$ROOT_DIR/run_py.sh" headless; then
    log_test_ok "$name"
    return 0
  else
    log_test_fail "$name"
    return 1
  fi
}

kill_our_server_on_port() {
  local port="$1"
  # Best-effort: requires lsof
  if ! command -v lsof >/dev/null 2>&1; then
    return 0
  fi
  # Find listeners on port
  local pids
  pids=$(lsof -nP -iTCP:"$port" -sTCP:LISTEN -t 2>/dev/null | tr '\n' ' ')
  for pid in $pids; do
    # Only kill our own server (serve.mjs)
    local cmd
    cmd=$(ps -p "$pid" -o command= 2>/dev/null || true)
    if printf '%s' "$cmd" | grep -q "platforms/web/healthcheck/serve.mjs"; then
      kill "$pid" >/dev/null 2>&1 || true
      # Wait up to ~2s for it to exit
      for _ in 1 2 3 4 5 6 7 8; do
        if ! ps -p "$pid" >/dev/null 2>&1; then break; fi
        sleep 0.25
      done
      if ps -p "$pid" >/dev/null 2>&1; then
        kill -9 "$pid" >/dev/null 2>&1 || true
      fi
    fi
  done
}

port_in_use_by_other() {
  local port="$1"
  if ! command -v lsof >/dev/null 2>&1; then
    return 1
  fi
  local pids
  pids=$(lsof -nP -iTCP:"$port" -sTCP:LISTEN -t 2>/dev/null | tr '\n' ' ')
  for pid in $pids; do
    local cmd
    cmd=$(ps -p "$pid" -o command= 2>/dev/null || true)
    if ! printf '%s' "$cmd" | grep -q "platforms/web/healthcheck/serve.mjs"; then
      # In use by a different process
      printf '%s' "$cmd"
      return 0
    fi
  done
  return 1
}

start_static_server() {
  local dir="$1"; local port="$2"
  # Kill stale instance of our Node server if present
  kill_our_server_on_port "$port"
  # Refuse to start if another (non-our) process is listening
  local offender
  offender=$(port_in_use_by_other "$port" || true)
  if [ -n "$offender" ]; then
    echo "Port $port is in use by another process: $offender" >&2
    echo "Hint: set PORT to a free port (e.g., PORT=8876) or stop the process above." >&2
    return 1
  fi
  # Use Node COOP/COEP server to enable SharedArrayBuffer/WebGPU readbacks
  PORT="$port" node "$ROOT_DIR/platforms/web/healthcheck/serve.mjs" >/dev/null 2>&1 &
  echo $!
}

wait_for_http() {
  local url="$1"; local attempts=0
  until curl -sSf "$url" >/dev/null 2>&1; do
    attempts=$((attempts+1))
    if [ "$attempts" -gt 60 ]; then
      return 1
    fi
    sleep 0.25
  done
  return 0
}

ensure_playwright() {
  # Install Playwright in the healthcheck folder if not available
  local dir="$ROOT_DIR/platforms/web/healthcheck"
  ( cd "$dir" && npm i --no-audit --no-fund --no-progress playwright@^1 >/dev/null 2>&1 || true )
  ( cd "$dir" && npx playwright install chromium >/dev/null 2>&1 || true )
}

run_web() {
  local name="platforms.web.healthcheck"
  # Build the web WASM package first to ensure the latest sources are used.
  if ! bash "$ROOT_DIR/build_web.sh"; then
    echo "Failed to build web package" >&2
    log_test_fail "$name"
    return 1
  fi
  # Reuse existing WASM pkg instead of rebuilding to keep healthcheck fast.
  # Expect that `build_web.sh` has been run previously when developing.
  local pkg_dir="$ROOT_DIR/platforms/web/pkg"
  if [ ! -d "$pkg_dir" ] || ! ls "$pkg_dir"/*.wasm >/dev/null 2>&1; then
    echo "WASM pkg not found (expected at $pkg_dir). Run build_web.sh once before healthchecks." >&2
    log_test_fail "$name"
    return 1
  fi

  # Ensure healthcheck has a fresh copy of the pkg without rebuilding.
  local hc_pkg="$ROOT_DIR/platforms/web/healthcheck/pkg"
  mkdir -p "$hc_pkg"
  rsync -a --delete "$pkg_dir/" "$hc_pkg/" 2>/dev/null || cp -a "$pkg_dir/." "$hc_pkg/"

  # Start static server and run Playwright against it
  local pid
  pid=$(start_static_server "$ROOT_DIR/platforms/web" "$PORT")
  srv_status=$?
  # Always cleanup the server if we started one
  cleanup() { if [ -n "${pid:-}" ] && ps -p "$pid" >/dev/null 2>&1; then kill "$pid" >/dev/null 2>&1 || true; fi; }
  trap cleanup EXIT
  if [ "$srv_status" -ne 0 ]; then
    cleanup
    log_test_fail "$name"
    trap - EXIT
    return 1
  fi

  if ! wait_for_http "http://localhost:$PORT/healthcheck/index.html"; then
    cleanup
    log_test_fail "$name"
    trap - EXIT
    return 1
  fi

  if ! command -v node >/dev/null 2>&1; then
    echo "Node.js is required to run the web healthcheck." >&2
    cleanup
    log_test_fail "$name"
    trap - EXIT
    return 1
  fi

  ensure_playwright
  if node "$ROOT_DIR/platforms/web/healthcheck/playwright.mjs" "http://localhost:$PORT/healthcheck/"; then
    cleanup
    trap - EXIT
    log_test_ok "$name"
    return 0
  else
    cleanup
    trap - EXIT
    log_test_fail "$name"
    return 1
  fi
}

main() {
  local mode; mode=$(parse_mode "${1:-all}")
  local passed=0; local failed=0; local total=0
  case "$mode" in
    py)
      echo "running 1 test"
      total=1
      if run_py; then passed=$((passed+1)); else failed=$((failed+1)); fi
      ;;
    web)
      echo "running 1 test"
      total=1
      if run_web; then passed=$((passed+1)); else failed=$((failed+1)); fi
      ;;
    all)
      echo "running 2 tests"
      total=2
      if run_py; then passed=$((passed+1)); else failed=$((failed+1)); fi
      if run_web; then passed=$((passed+1)); else failed=$((failed+1)); fi
      ;;
  esac

  if [ "$failed" -eq 0 ]; then
    printf "\n%btest result: ok%b. %d passed; %d failed\n" "$GREEN" "$RESET" "$passed" "$failed"
    exit 0
  else
    printf "\n%btest result: FAILED%b. %d passed; %d failed\n" "$RED" "$RESET" "$passed" "$failed"
    exit 1
  fi
}

main "$@"

