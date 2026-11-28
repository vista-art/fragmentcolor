# shellcheck shell=bash
# Process helpers the tools source.

# Stops a process and every process it started.
# pnpm and node shims such as Volta's run the real server as a child,
# and a SIGTERM to the parent alone leaves that server holding its port.
stop_tree() {
  local child
  for child in $(pgrep -P "$1" 2>/dev/null); do
    stop_tree "$child"
  done
  kill "$1" 2>/dev/null || true
}
