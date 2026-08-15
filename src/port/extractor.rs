//! The extraction port.

use std::future::Future;

use crate::{BackendInfo, CanonicalRealm, ExtractScope, Result};

/// Pulls one backend's configuration state into the canonical model.
///
/// # Contract
///
/// - `detect` identifies the backend and the schema version visible at the
///   source. It must be safe to call and cheap.
/// - `extract` returns one realm's worth of state. Collections must be sorted
///   by a stable key (id, or a natural composite key), native content must be
///   produced deterministically (stable object keys, fixed number forms), and
///   every field the backend's [`FieldManifest`](crate::FieldManifest) marks
///   volatile must be placed in [`CanonicalRealm::volatile`], never in the
///   config body. The [`testing`](crate::testing) contract suite enforces the
///   parts of this that need no live database.
///
/// # Connection ownership
///
/// There is deliberately no connection handle in this port. Each adapter owns
/// its own connection pool, built from its own constructor configuration, so
/// core carries zero database knowledge and no runtime dependency.
///
/// # Dispatch
///
/// This trait is **not object-safe** and must not be used through
/// `Box<dyn Extractor>` — a limitation of native async methods in traits, and
/// a deliberate one (see `ARCHITECTURE.md` §2.1). The CLI dispatches via a
/// concrete enum wrapping each adapter type with a `match`. The returned
/// futures are `Send`, so the CLI can drive adapters on a tokio runtime.
///
/// The explicit `impl Future + Send` return positions (rather than `async fn`)
/// are what make the `Send` guarantee part of the contract.
#[allow(clippy::manual_async_fn)]
pub trait Extractor: Send + Sync {
    /// Identify the backend and the schema version detected at the source.
    fn detect(&self) -> impl Future<Output = Result<BackendInfo>> + Send;

    /// Pull one canonical realm's worth of state from the backend.
    fn extract(&self, scope: &ExtractScope) -> impl Future<Output = Result<CanonicalRealm>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BackendInfo;
    use crate::version::Version;
    use std::{
        future::Future,
        pin::pin,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    // A dependency-free block_on so the core's no-tokio rule holds in tests.
    fn block_on<F: Future>(fut: F) -> F::Output {
        const VTABLE: RawWakerVTable = RawWakerVTable::new(|_| RAW_WAKER, |_| {}, |_| {}, |_| {});
        const RAW_WAKER: RawWaker = RawWaker::new(std::ptr::null(), &VTABLE);
        let waker = unsafe { Waker::from_raw(RAW_WAKER) };
        let mut cx = Context::from_waker(&waker);
        let mut fut = pin!(fut);
        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(output) => break output,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    struct Stub {
        info: BackendInfo,
    }

    impl Extractor for Stub {
        async fn detect(&self) -> Result<BackendInfo> {
            Ok(self.info.clone())
        }

        async fn extract(&self, _scope: &ExtractScope) -> Result<CanonicalRealm> {
            Ok(CanonicalRealm::default())
        }
    }

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn extractors_are_send_and_sync() {
        assert_send_sync::<Stub>();
    }

    #[test]
    fn extractor_futures_resolve_without_a_runtime() {
        let stub = Stub {
            info: BackendInfo {
                backend: "stub".into(),
                detected_version: Version::new(1, 0, 0),
                extra: Default::default(),
            },
        };
        let detected = block_on(stub.detect()).unwrap();
        assert_eq!(detected.backend, "stub");
        let realm = block_on(stub.extract(&ExtractScope::new("test"))).unwrap();
        assert!(realm.clients.is_empty());
    }
}
