// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

//! Shallow wrapper around [`tokio::sync::watch`] primitives with an
//! opinionated API comprising of more recognizable function names
//! and hiding of unneeded features.

#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]

use std::{ops::Deref, sync::Arc};

use tokio::sync::watch;

use super::OrphanedSubscriberError;

#[derive(Debug)]
pub struct Ref<'r, T>(watch::Ref<'r, T>);

impl<'r, T> AsRef<T> for Ref<'r, T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<'r, T> Deref for Ref<'r, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[must_use]
fn publisher_has_subscribers<T>(watch_tx: &Arc<watch::Sender<T>>) -> Option<bool> {
    if !watch_tx.is_closed() {
        return Some(true);
    }
    debug_assert!(Arc::strong_count(watch_tx) >= 1);
    debug_assert_eq!(Arc::weak_count(watch_tx), 0);
    if Arc::strong_count(watch_tx) == 1 {
        // No other publisher exists from which new subscribers
        // could be created concurrently. This would otherwise
        // cause race conditions when the channel is reopened.
        return Some(false);
    }
    None
}

#[must_use]
#[inline]
fn publisher_subscribe<T>(watch_tx: &watch::Sender<T>) -> Subscriber<T> {
    Subscriber {
        rx: watch_tx.subscribe(),
        changed: true,
    }
}

#[must_use]
#[inline]
fn publisher_read<T>(watch_tx: &watch::Sender<T>) -> Ref<'_, T> {
    Ref(watch_tx.borrow())
}

#[derive(Debug)]
pub struct ReadOnlyPublisher<T> {
    tx: Arc<watch::Sender<T>>,
}

impl<T> Clone for ReadOnlyPublisher<T> {
    fn clone(&self) -> Self {
        Self {
            tx: Arc::clone(&self.tx),
        }
    }
}

impl<T> ReadOnlyPublisher<T> {
    #[must_use]
    pub fn has_subscribers(&self) -> Option<bool> {
        publisher_has_subscribers(&self.tx)
    }

    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        publisher_subscribe(&self.tx)
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, T> {
        publisher_read(&self.tx)
    }
}

#[derive(Debug)]
pub struct Publisher<T> {
    tx: Arc<watch::Sender<T>>,
}

impl<T> Publisher<T> {
    #[must_use]
    pub fn new(initial_value: T) -> Self {
        Self {
            tx: Arc::new(watch::channel(initial_value).0),
        }
    }

    #[must_use]
    pub fn clone_read_only(&self) -> ReadOnlyPublisher<T> {
        ReadOnlyPublisher {
            tx: Arc::clone(&self.tx),
        }
    }

    #[must_use]
    pub fn has_subscribers(&self) -> Option<bool> {
        publisher_has_subscribers(&self.tx)
    }

    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        publisher_subscribe(&self.tx)
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, T> {
        publisher_read(&self.tx)
    }

    pub fn write(&self, new_value: impl Into<T>) {
        // Sender::send() would prematurely abort and fail if
        // no senders are connected and the current value would
        // not be replaced as expected. Therefore we have to use
        // Sender::send_modify() here!
        self.tx.send_modify(move |value| *value = new_value.into());
    }

    #[must_use]
    pub fn replace(&self, new_value: impl Into<T>) -> T {
        self.tx.send_replace(new_value.into())
    }

    pub fn modify<M>(&self, modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool,
    {
        self.tx.send_if_modified(modify)
    }
}

impl<T> Default for Publisher<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[derive(Debug)]
pub struct Subscriber<T> {
    rx: watch::Receiver<T>,
    // TODO: Remove this flag after upgrading to Tokio v1.33 and
    // instead use Receiver::mark_changed() to enforce that the
    // initial value is considered as changed.
    // See also: <https://github.com/tokio-rs/tokio/pull/6014>
    changed: bool,
}

impl<T> Clone for Subscriber<T> {
    fn clone(&self) -> Self {
        let Self { rx, changed: _ } = self;
        Self {
            rx: rx.clone(),
            // The state of the cloned subscriber must also be considered as changed!
            changed: true,
        }
    }
}

impl<T> Subscriber<T> {
    #[must_use]
    pub fn read(&self) -> Ref<'_, T> {
        Ref(self.rx.borrow())
    }

    #[must_use]
    pub fn read_ack(&mut self) -> Ref<'_, T> {
        self.changed = false;
        Ref(self.rx.borrow_and_update())
    }

    pub async fn read_ack_filtered(
        &mut self,
        filter_fn: impl FnMut(&T) -> bool,
    ) -> Result<Ref<'_, T>, OrphanedSubscriberError> {
        self.changed = false;
        self.rx
            .wait_for(filter_fn)
            .await
            .map(Ref)
            .map_err(|_| OrphanedSubscriberError)
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn changed(&mut self) -> Result<(), OrphanedSubscriberError> {
        let Self { changed, rx } = self;
        if *changed {
            *changed = false;
            return Ok(());
        }
        rx.changed().await.map_err(|_| OrphanedSubscriberError)
    }

    #[cfg(feature = "async-stream")]
    pub fn into_stream<U>(self, capture_fn: impl FnMut(&T) -> U) -> impl futures::Stream<Item = U> {
        self.into_filtered_stream(|_| true, capture_fn)
    }

    #[cfg(feature = "async-stream")]
    pub fn into_filtered_stream<U>(
        self,
        mut filter_fn: impl FnMut(&T) -> bool,
        mut capture_fn: impl FnMut(&T) -> U,
    ) -> impl futures::Stream<Item = U> {
        async_stream::stream! {
            let mut this = self;
            let filter_fn = &mut filter_fn;
            loop {
                match this.read_ack_filtered(|v| filter_fn(v)).await.map(|v| capture_fn(&v)) {
                    Ok(captured) => {
                        yield captured;
                    }
                    Err(OrphanedSubscriberError) => {
                        // Stream exhausted after publisher disappeared
                        return;
                    }
                }
            }
        }
    }

    #[cfg(feature = "async-stream")]
    pub fn into_filtered_stream_or_defer<U, R>(
        self,
        mut filter_fn: impl FnMut(&T) -> bool,
        mut capture_or_defer_fn: impl FnMut(&T) -> Result<U, R>,
    ) -> impl futures::Stream<Item = U>
    where
        R: std::future::Future<Output = ()>,
    {
        async_stream::stream! {
            let mut this = self;
            // Defer forever
            let mut next_defer = futures::future::Either::Left(std::future::pending());
            loop {
                futures::select! {
                    _ = futures::FutureExt::fuse(next_defer) => {
                        // Defer forever
                        next_defer = futures::future::Either::Left(std::future::pending());
                    }
                    next_ref = futures::FutureExt::fuse(this.read_ack_filtered(|v| filter_fn(v))) => {
                        let Ok(next_ref) = next_ref else {
                            // Stream exhausted after publisher disappeared
                            return;
                        };
                        let captured_or_defer = capture_or_defer_fn(&next_ref);
                        // Release the read lock before handling the result!
                        drop(next_ref);
                        match captured_or_defer {
                            Ok(captured) => {
                                yield captured;
                                // Defer forever
                                next_defer = futures::future::Either::Left(std::future::pending());
                            }
                            Err(defer) => {
                                // Don't yield and defer instead
                                next_defer = futures::future::Either::Right(defer);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Publisher;

    #[tokio::test]
    async fn changed_after_subscribing() {
        let tx = Publisher::new(0);
        let mut rx = tx.subscribe();
        // The initial value is considered as changed.
        tokio::select! {
            _ = rx.changed() => (),
            _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => panic!("not changed as expected"),
        }
    }

    #[tokio::test]
    async fn changed_after_cloning_subscriber() {
        let tx = Publisher::new(0);
        let mut rx = tx.subscribe();
        assert_eq!(*tx.read(), *rx.read_ack());
        // No longer changed after acknowledging the value.
        tokio::select! {
            _ = rx.changed() => panic!("unexpected change"),
            _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => (),
        }
        // The initial value of the cloned subscriber is considered as changed,
        // even if the original publisher has already acknowledged the value
        // before cloning it.
        let mut rx_cloned = rx.clone();
        tokio::select! {
            _ = rx_cloned.changed() => (),
            _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => panic!("not changed as expected"),
        }
    }

    // This test won't terminate if the value is not considered as changed as expected.
    // after reading but not acknowledging it.
    #[tokio::test]
    async fn changed_after_reading_without_acknowledging() {
        let tx = Publisher::new(0);
        let mut rx = tx.subscribe();
        // Read but don't acknowledge the value.
        assert_eq!(*tx.read(), *rx.read());
        // Still changed after reading but not acknowledging the value.
        tokio::select! {
            _ = rx.changed() => (),
            _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => panic!("not changed as expected"),
        }
        assert_eq!(*tx.read(), *rx.read_ack());
        // No longer changed after acknowledging the value.
        tokio::select! {
            _ = rx.changed() => panic!("unexpected change"),
            _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => (),
        }
    }
}

#[cfg(test)]
mod traits {
    use async_trait::async_trait;

    use super::{Publisher, ReadOnlyPublisher, Ref, Subscriber};
    use crate::OrphanedSubscriberError;

    // <https://github.com/rust-lang/api-guidelines/issues/223#issuecomment-683346783>
    const _: () = {
        const fn assert_send<T: Send>() {}
        let _ = assert_send::<Publisher<i32>>;
    };

    // <https://github.com/rust-lang/api-guidelines/issues/223#issuecomment-683346783>
    const _: () = {
        const fn assert_sync<T: Sync>() {}
        let _ = assert_sync::<Publisher<i32>>;
    };

    impl<T> crate::traits::Ref<T> for Ref<'_, T> {}

    impl<'r, T> crate::traits::Subscribable<'r, T, Ref<'r, T>, Subscriber<T>> for ReadOnlyPublisher<T> {
        fn has_subscribers(&self) -> Option<bool> {
            self.has_subscribers()
        }

        fn subscribe(&self) -> Subscriber<T> {
            self.subscribe()
        }
    }

    impl<'r, T> crate::traits::ReadOnlyPublisher<'r, T, Ref<'r, T>, Subscriber<T>>
        for ReadOnlyPublisher<T>
    {
    }

    impl<'r, T> crate::traits::Readable<'r, T, Ref<'r, T>> for ReadOnlyPublisher<T> {
        fn read(&self) -> Ref<'_, T> {
            self.read()
        }
    }

    impl<'r, T> crate::traits::Subscribable<'r, T, Ref<'r, T>, Subscriber<T>> for Publisher<T> {
        fn has_subscribers(&self) -> Option<bool> {
            self.has_subscribers()
        }

        fn subscribe(&self) -> Subscriber<T> {
            self.subscribe()
        }
    }

    impl<'r, T> crate::traits::Publisher<'r, T, Ref<'r, T>, Subscriber<T>, ReadOnlyPublisher<T>>
        for Publisher<T>
    {
        fn clone_read_only(&self) -> ReadOnlyPublisher<T> {
            self.clone_read_only()
        }

        fn write(&self, new_value: impl Into<T>) {
            self.write(new_value);
        }

        fn replace(&self, new_value: impl Into<T>) -> T {
            self.replace(new_value)
        }

        fn modify<M>(&self, modify: M) -> bool
        where
            M: FnOnce(&mut T) -> bool,
        {
            self.modify(modify)
        }
    }

    impl<'r, T> crate::traits::Readable<'r, T, Ref<'r, T>> for Publisher<T> {
        fn read(&self) -> Ref<'_, T> {
            self.read()
        }
    }

    impl<'r, T> crate::traits::Readable<'r, T, Ref<'r, T>> for Subscriber<T> {
        fn read(&self) -> Ref<'_, T> {
            self.read()
        }
    }

    impl<'r, T> crate::traits::Subscriber<'r, T, Ref<'r, T>> for Subscriber<T> {
        fn read_ack(&mut self) -> Ref<'_, T> {
            self.read_ack()
        }
    }

    #[async_trait]
    impl<T> crate::traits::ChangeListener for Subscriber<T>
    where
        T: Send + Sync,
    {
        async fn changed(&mut self) -> Result<(), OrphanedSubscriberError> {
            self.changed().await
        }
    }
}
