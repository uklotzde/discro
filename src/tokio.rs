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
use crate::subscriber::{filter_map_changed, map_changed};

#[derive(Debug)]
pub struct Ref<'r, T>(watch::Ref<'r, T>);

impl<'r, T> Deref for Ref<'r, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
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
    Subscriber::new(watch_tx.subscribe())
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

    pub fn write(&self, new_value: T) {
        // Sender::send() would prematurely abort and fail if
        // no senders are connected and the current value would
        // not be replaced as expected. Therefore we have to use
        // Sender::send_modify() here!
        // The conversion into the value is done before the locking scope.
        self.tx.send_modify(move |value| *value = new_value);
    }

    #[must_use]
    pub fn replace(&self, new_value: T) -> T {
        self.tx.send_replace(new_value)
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
    // TODO: Remove the changed flag after upgrading to Tokio v1.33.0
    changed: bool,
}

impl<T> Subscriber<T> {
    fn new(mut rx: watch::Receiver<T>) -> Self {
        let changed = rx.borrow_and_update().has_changed();
        Self { rx, changed }
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, T> {
        Ref(self.rx.borrow())
    }

    #[must_use]
    pub fn read_ack(&mut self) -> Ref<'_, T> {
        if self.changed {
            self.changed = false;
        }
        Ref(self.rx.borrow_and_update())
    }

    pub fn mark_changed(&mut self) {
        //FIXME: Requires Tokio v1.33.0
        //self.rx.mark_changed();
        self.changed = true;
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn changed(&mut self) -> Result<(), OrphanedSubscriberError> {
        if self.changed {
            self.changed = false;
            return Ok(());
        }
        self.rx.changed().await.map_err(|_| OrphanedSubscriberError)
    }

    pub async fn read_changed(&mut self) -> Result<Ref<'_, T>, OrphanedSubscriberError> {
        self.changed().await.map(|()| self.read_ack())
    }

    pub async fn map_changed<U>(
        &mut self,
        map_fn: impl FnMut(&T) -> U,
    ) -> Result<U, OrphanedSubscriberError> {
        map_changed(self, map_fn).await
    }

    pub async fn filter_map_changed<U>(
        &mut self,
        filter_map_fn: impl FnMut(&T) -> Option<U>,
    ) -> Result<U, OrphanedSubscriberError> {
        filter_map_changed(self, filter_map_fn).await
    }
}

impl<T> Clone for Subscriber<T> {
    fn clone(&self) -> Self {
        let Self { rx, changed } = self;
        Self {
            rx: rx.clone(),
            changed: *changed,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Publisher;

    // This test won't terminate if the value is not considered as changed as expected.
    // after reading but not acknowledging it.
    #[tokio::test]
    async fn changed_after_reading_without_acknowledging() {
        let tx = Publisher::new(0);
        let mut rx = tx.subscribe();
        // Publish a new, changed value
        tx.write(0);
        // Read but don't acknowledge the value.
        assert_eq!(*tx.read(), *rx.read());
        // Still changed after reading but not acknowledging the value.
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(1), rx.changed())
                .await
                .is_ok()
        );
        assert_eq!(*tx.read(), *rx.read_ack());
        // No longer changed after acknowledging the value.
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(1), rx.changed())
                .await
                .is_err()
        );
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

        fn write(&self, new_value: T) {
            self.write(new_value);
        }

        fn replace(&self, new_value: T) -> T {
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
        fn mark_changed(&mut self) {
            self.mark_changed();
        }

        async fn changed(&mut self) -> Result<(), OrphanedSubscriberError> {
            self.changed().await
        }
    }
}
