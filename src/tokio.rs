// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

//! Shallow wrapper around [`tokio::sync::watch`] primitives with an
//! opinionated API comprising of more recognizable function names
//! and hiding of unneeded features.

#![allow(missing_docs)]

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
    pub fn has_subscribers(&self) -> bool {
        !self.tx.is_closed()
    }

    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        Subscriber {
            rx: self.tx.subscribe(),
        }
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, T> {
        Ref(self.tx.borrow())
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
    pub fn has_subscribers(&self) -> bool {
        !self.tx.is_closed()
    }

    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        Subscriber {
            rx: self.tx.subscribe(),
        }
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, T> {
        Ref(self.tx.borrow())
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
}

impl<T> Clone for Subscriber<T> {
    fn clone(&self) -> Self {
        Self {
            rx: self.rx.clone(),
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
        Ref(self.rx.borrow_and_update())
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn changed(&mut self) -> Result<(), OrphanedSubscriberError> {
        self.rx.changed().await.map_err(|_| OrphanedSubscriberError)
    }

    #[cfg(feature = "async-stream")]
    pub fn into_stream<U>(
        self,
        mut capture_fn: impl FnMut(&T) -> U,
    ) -> impl futures::Stream<Item = U> {
        async_stream::stream! {
            let mut this = self;
            loop {
                let captured = capture_fn(this.read_ack().as_ref());
                yield captured;
                if this.changed().await.is_err() {
                    // Stream exhausted after publisher disappeared
                    return;
                }
            }
        }
    }

    #[cfg(feature = "async-stream")]
    pub fn into_stream_or_skip<U>(
        self,
        mut capture_or_skip_fn: impl FnMut(&T) -> Option<U>,
    ) -> impl futures::Stream<Item = U> {
        async_stream::stream! {
            let mut this = self;
            loop {
                let Some(captured) = capture_or_skip_fn(this.read_ack().as_ref()) else {
                    // Skip value
                    continue;
                };
                yield captured;
                if this.changed().await.is_err() {
                    // Stream exhausted after publisher disappeared
                    return;
                }
            }
        }
    }

    #[cfg(feature = "async-stream")]
    pub fn into_stream_or_defer<U, R>(
        self,
        mut capture_or_defer_fn: impl FnMut(&T) -> Result<U, R>,
    ) -> impl futures::Stream<Item = U>
    where
        R: std::future::Future<Output = ()>,
    {
        async_stream::stream! {
            let mut this = self;
            loop {
                let defer = match capture_or_defer_fn(this.read_ack().as_ref()) {
                    Ok(captured) => {
                        yield captured;
                        // Unconditionally wait for the next change notification,
                        // i.e. defer forever.
                        futures::future::Either::Left(std::future::pending())
                    }
                    Err(defer) => {
                        // Don't yield and defer instead.
                        futures::future::Either::Right(defer)
                    }
                };
                futures::select! {
                    _ = futures::FutureExt::fuse(defer) => (),
                    changed = futures::FutureExt::fuse(this.changed()) => {
                        if changed.is_err() {
                            // Stream exhausted after publisher disappeared
                            return;
                        }
                    }
                }
            }
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
        fn has_subscribers(&self) -> bool {
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
        fn has_subscribers(&self) -> bool {
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
