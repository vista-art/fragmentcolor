#!/usr/bin/env bash
set -euo pipefail
set -x

# Build Python wheels with maturin. If on macOS and CODESIGN_IDENTITY is set,
# optionally codesign native extension(s) inside the built wheel(s) while keeping
# RECORD accurate by repacking with the 'wheel' tool.
#
# Usage:
#   ./build_py.sh            # release build
#   CODESIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)" ./build_py.sh
#
# Requirements:
#   - maturin installed in PATH (pipx install maturin)
#   - On macOS for signing: Xcode command line tools, codesign, and Python 'wheel'

OUT_DIR="dist"

# Build wheels
maturin build --release --out "$OUT_DIR"

# Best-effort codesign on macOS
UNAME_OUT="$(uname -s)" || UNAME_OUT=""
if [[ "$UNAME_OUT" == "Darwin" ]] && [[ -n "${CODESIGN_IDENTITY:-}" ]]; then
  # Ensure the 'wheel' tool is available for unpack/pack
  if ! python3 -c 'import wheel' >/dev/null 2>&1; then
    python3 -m pip install --user wheel
  fi

  mkdir -p "$OUT_DIR-signed"
  for whl in "$OUT_DIR"/*.whl; do
    [[ -f "$whl" ]] || continue
    tmpdir="$(mktemp -d)"
    # Unpack the wheel so we can sign native libraries inside
    python3 -m wheel unpack "$whl" -d "$tmpdir"
    unpacked_dir="$(find "$tmpdir" -mindepth 1 -maxdepth 1 -type d | head -n1)"

    # Sign all native extensions inside the wheel
    while IFS= read -r -d '' sofile; do
      codesign --force --timestamp --options=runtime --sign "$CODESIGN_IDENTITY" "$sofile"
    done < <(find "$unpacked_dir" -type f \( -name "*.so" -o -name "*.dylib" \) -print0)

    # Repack to regenerate the RECORD entries
    python3 -m wheel pack "$unpacked_dir" -d "$OUT_DIR-signed"
    rm -rf "$tmpdir"
  done

  # Replace original wheels with signed ones (if any)
  if compgen -G "$OUT_DIR-signed/*.whl" >/dev/null; then
    rm -f "$OUT_DIR"/*.whl
    mv "$OUT_DIR-signed"/*.whl "$OUT_DIR"/
    rmdir "$OUT_DIR-signed" || true
  else
    echo "No signed wheels produced; leaving originals intact"
  fi
fi
