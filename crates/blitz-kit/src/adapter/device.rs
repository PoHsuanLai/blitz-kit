//! Opening a device on a ranked adapter: what every consumer's GPU setup shares.

use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

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
    let waker = Waker::from(Arc::new(Unpark(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => thread::park(),
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

/// A device and queue on `adapter`, labelled `label`, with the optional features the renderers
/// use (`CLEAR_TEXTURE`, `PIPELINE_CACHE`) when the adapter has them.
pub fn request_device(
    adapter: &wgpu::Adapter,
    label: &str,
) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    let features =
        adapter.features() & (wgpu::Features::CLEAR_TEXTURE | wgpu::Features::PIPELINE_CACHE);
    block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some(label),
        required_features: features,
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        ..Default::default()
    }))
}
