// @TODO catalog all panics. Glob search "TECH DEBT"
//       and move them all here.
//       This will catch the most important ones.
//       Then glob search ".expect", ".unwrap", and ".panic"

pub(crate) const WINDOW_FAILED_TO_ACQUIRE_READ_LOCK: &str = "Failed to read Window state!";
pub(crate) const WINDOW_FAILED_TO_ACQUIRE_WRITE_LOCK: &str = "Failed to write Window state!";

pub(crate) const SCENE_FAILED_TO_ACQUIRE_READ_LOCK: &str = "Failed to read Scene state!";
pub(crate) const SCENE_FAILED_TO_ACQUIRE_WRITE_LOCK: &str = "Failed to write Scene state!";

pub(crate) const DEFAULT_IMAGE_NOT_FOUND: &str = "Default image does not exist!";
