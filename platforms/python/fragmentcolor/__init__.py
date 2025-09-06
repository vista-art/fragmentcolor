import sys
from pathlib import Path

try:
    from .fragmentcolor import *  # type: ignore
    # Keep __doc__ and __all__ aligned with the extension module
    from . import fragmentcolor as _fragmentcolor  # type: ignore
    __doc__ = _fragmentcolor.__doc__
    if hasattr(_fragmentcolor, "__all__"):
        __all__ = _fragmentcolor.__all__
except OSError as e:
    # Provide a helpful hint for macOS code signature errors during import
    if sys.platform == "darwin" and "code signature in" in str(e) and "not valid for use in process" in str(e):
        pkg_dir = Path(__file__).parent
        candidates = list(pkg_dir.glob("fragmentcolor*.so")) + \
            list(pkg_dir.glob("fragmentcolor*.dylib"))
        hint = str(candidates[0]) if candidates else str(
            pkg_dir / "fragmentcolor.abi3.so")
        print("\nMacOS Code Signature Issue Detected!\n")
        print("To resolve this, please clear extended attributes on the extension module, e.g.:")
        print(f"xattr -c {hint}\n")
        print("Then try importing fragmentcolor again.\n")
    raise
