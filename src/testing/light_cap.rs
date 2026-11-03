//! A light cap for the current test thread alone, so a test that forces a
//! small device cap leaves the tests running beside it alone.

use std::cell::Cell;

thread_local! {
    static CAP: Cell<u32> = const { Cell::new(u32::MAX) };
}

/// Lowers the light cap for the calling test thread only.
pub(crate) fn lower(cap: u32) {
    CAP.with(|current| current.set(current.get().min(cap)));
}

/// The light cap the calling test thread set, or `u32::MAX`.
pub(crate) fn read() -> u32 {
    CAP.with(Cell::get)
}
