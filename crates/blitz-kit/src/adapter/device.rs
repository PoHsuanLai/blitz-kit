//! Opening a device on a ranked adapter: what every consumer's GPU setup shares.

use std::fmt;
use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

use super::{AdapterFacts, AdapterPref, rank};

/// The environment variable that names the adapter, overriding any [`AdapterPref`] (read by the
/// consumer, which passes the value to [`AdapterPref::with_env`]).
pub const ADAPTER_ENV: &str = "WGPU_ADAPTER_NAME";

struct Unpark(Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Drive `future` to completion on this thread: wgpu's adapter and device futures resolve
/// without an executor on the native backends, so parking until woken is all that is needed.
pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    let mut future = pin!(future);
    loop {
        if let Some(value) = drive(future.as_mut(), None) {
            return value;
        }
    }
}

/// Like [`block_on`], but gives up when `deadline` has passed: `None` for a future that is still
/// pending then. For the calls a hung driver can leave pending forever.
pub fn block_on_timeout<T>(future: impl Future<Output = T>, deadline: Duration) -> Option<T> {
    drive(pin!(future), Some(Instant::now() + deadline))
}

/// Poll `future` and park between polls: until woken when `end` is `None`, until `end` otherwise
/// (`None` result: `end` passed with the future still pending).
fn drive<T>(mut future: Pin<&mut impl Future<Output = T>>, end: Option<Instant>) -> Option<T> {
    let waker = Waker::from(Arc::new(Unpark(thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return Some(value);
        }
        match end {
            None => thread::park(),
            Some(end) => thread::park_timeout(end.checked_duration_since(Instant::now())?),
        }
    }
}

/// `adapters` in the order [`rank`] gives under `pref` (the effective preference, the
/// environment already applied).
pub fn ranked(adapters: Vec<wgpu::Adapter>, pref: &AdapterPref) -> Vec<wgpu::Adapter> {
    let facts: Vec<AdapterFacts> = adapters.iter().map(AdapterFacts::of).collect();
    let mut slots: Vec<Option<wgpu::Adapter>> = adapters.into_iter().map(Some).collect();
    rank(pref, &facts)
        .into_iter()
        .filter_map(|i| slots[i].take())
        .collect()
}

/// Whether a device still works, as the callbacks wgpu runs on its own threads last left it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Working,
    /// The driver reset or the device was destroyed: every resource made on it is gone, and the
    /// consumer has to make a new device.
    Lost,
}

const WORKING: u8 = 0;
const LOST: u8 = 1;

/// A device's health, shared with its device-lost callback. Clones see the same value.
#[derive(Debug, Clone, Default)]
pub struct DeviceHealth(Arc<AtomicU8>);

impl DeviceHealth {
    /// Whether the device still works.
    pub fn state(&self) -> Health {
        match self.0.load(Ordering::Acquire) {
            WORKING => Health::Working,
            _ => Health::Lost,
        }
    }

    fn lose(&self) {
        self.0.store(LOST, Ordering::Release);
    }
}

/// How many wgpu errors one device logs before it goes quiet: a validation error that repeats
/// on every frame must not flood the journal.
const MAX_LOGGED_ERRORS: u32 = 16;

/// Whether the error after `logged` earlier ones is still worth a line.
fn worth_logging(logged: u32) -> bool {
    logged < MAX_LOGGED_ERRORS
}

/// A device and queue, and the health the device-lost callback keeps.
#[derive(Debug)]
pub struct WatchedDevice {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub health: DeviceHealth,
}

/// Why a device could not be opened.
#[derive(Debug)]
pub enum DeviceError {
    /// The adapter refused.
    Refused(wgpu::RequestDeviceError),
    /// The driver gave no answer within the deadline.
    NoAnswer { after: Duration },
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceError::Refused(error) => write!(f, "{error}"),
            DeviceError::NoAnswer { after } => {
                write!(f, "the driver gave no answer within {after:?}")
            }
        }
    }
}

impl std::error::Error for DeviceError {}

/// A device and queue on `adapter`, labelled `label`, with the optional features the renderers
/// use (`CLEAR_TEXTURE`, `PIPELINE_CACHE`) when the adapter has them.
///
/// wgpu's own handler for an error nobody catches panics; this device logs it instead (a
/// bounded number of lines), and a lost device is logged.
pub fn request_device(
    adapter: &wgpu::Adapter,
    label: &str,
) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    let (device, queue) = block_on(adapter.request_device(&descriptor(adapter, label)))?;
    watch(&device, label);
    Ok((device, queue))
}

/// [`request_device`] for a consumer that rebuilds on device loss: the device's [`DeviceHealth`]
/// comes with it, and the request gives up after `deadline`.
pub fn request_watched_device(
    adapter: &wgpu::Adapter,
    label: &str,
    deadline: Duration,
) -> Result<WatchedDevice, DeviceError> {
    let (device, queue) = block_on_timeout(
        adapter.request_device(&descriptor(adapter, label)),
        deadline,
    )
    .ok_or(DeviceError::NoAnswer { after: deadline })?
    .map_err(DeviceError::Refused)?;
    let health = watch(&device, label);
    Ok(WatchedDevice {
        device,
        queue,
        health,
    })
}

fn descriptor<'a>(adapter: &wgpu::Adapter, label: &'a str) -> wgpu::DeviceDescriptor<'a> {
    let features =
        adapter.features() & (wgpu::Features::CLEAR_TEXTURE | wgpu::Features::PIPELINE_CACHE);
    wgpu::DeviceDescriptor {
        label: Some(label),
        required_features: features,
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        ..Default::default()
    }
}

/// Install the error and device-lost callbacks on `device`.
fn watch(device: &wgpu::Device, label: &str) -> DeviceHealth {
    let health = DeviceHealth::default();
    let logged = Arc::new(AtomicU32::new(0));
    let name = label.to_owned();
    device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
        if worth_logging(logged.fetch_add(1, Ordering::Relaxed)) {
            eprintln!("{name}: GPU error: {error}");
        }
    }));
    let name = label.to_owned();
    let lost = health.clone();
    device.set_device_lost_callback(move |reason, message| {
        eprintln!("{name}: GPU device lost ({reason:?}): {message}");
        lost.lose();
    });
    health
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pending_future_gives_up_at_the_deadline_and_a_ready_one_does_not() {
        let started = Instant::now();
        let gave_up = block_on_timeout(std::future::pending::<u8>(), Duration::from_millis(30));
        assert_eq!(gave_up, None);
        assert!(
            started.elapsed() >= Duration::from_millis(30),
            "it waited out the deadline rather than failing at once"
        );
        assert_eq!(
            block_on_timeout(std::future::ready(7u8), Duration::ZERO),
            Some(7),
            "a future that is ready needs no time"
        );
    }

    #[test]
    fn a_lost_device_stays_lost_for_every_clone() {
        let health = DeviceHealth::default();
        let seen_by_the_consumer = health.clone();
        assert_eq!(seen_by_the_consumer.state(), Health::Working);
        health.lose();
        assert_eq!(seen_by_the_consumer.state(), Health::Lost);
    }

    #[test]
    fn only_the_first_errors_are_logged() {
        let cases: &[(&str, u32, bool)] = &[
            ("the first", 0, true),
            ("the last one allowed", MAX_LOGGED_ERRORS - 1, true),
            ("the next", MAX_LOGGED_ERRORS, false),
            ("long after", u32::MAX, false),
        ];
        for (name, logged, want) in cases {
            assert_eq!(worth_logging(*logged), *want, "{name}");
        }
    }
}
