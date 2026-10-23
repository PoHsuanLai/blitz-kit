//! One lock for every wgpu opener in the process. The Vulkan loader's ICD scan is not safe to run
//! twice at once: two threads that each make their first `wgpu::Instance`, or request a device,
//! at the same moment can crash in `loader_icd_scan` or `loader_get_icd_and_device`. Each
//! consumer that opened once per process behind a lock of its own still raced the others, so
//! they all take this one.

use std::sync::{Mutex, MutexGuard, PoisonError};

static OPENING: Mutex<()> = Mutex::new(());

/// Hold the returned guard while making a `wgpu::Instance`, enumerating its adapters and
/// requesting a device. The lock is not re-entrant: blitz-kit's own [`request_device`] and
/// [`request_watched_device`] do not take it, so a caller may hold it around them.
///
/// [`request_device`]: super::request_device
/// [`request_watched_device`]: super::request_watched_device
pub fn opening() -> MutexGuard<'static, ()> {
    OPENING.lock().unwrap_or_else(PoisonError::into_inner)
}
