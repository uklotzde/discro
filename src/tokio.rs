// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

//! Shallow wrapper around [`tokio::sync::watch`] primitives with an
//! opinionated API comprising of more recognizable function names
//! and hiding of unneeded features.

#![expect(missing_docs)]
#![expect(clippy::missing_errors_doc)]

use std::ops::Deref;

use tokio::sync::watch;

use crate::{
    ModifiedStatus, OrphanedSubscriberError,
    subscriber::{filter_map_changed, map_changed},
};

#[derive(Debug)]
pub struct Ref<'r, T>(watch::Ref<'r, T>);

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Publisher<T> {
    tx: watch::Sender<T>,
}

impl<T> Publisher<T> {
    #[must_use]
    pub fn new(initial_value: T) -> Self {
        Self {
            tx: watch::channel(initial_value).0,
        }
    }

    #[must_use]
    pub fn observe(&self) -> Observer<T> {
        Observer {
            tx: self.tx.clone(),
        }
    }

    #[must_use]
    pub fn has_subscribers(&self) -> bool {
        !self.tx.is_closed()
    }

    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        Subscriber::new(self.tx.subscribe())
    }

    #[must_use]
    pub fn subscribe_changed(&self) -> Subscriber<T> {
        let mut subscriber = self.subscribe();
        subscriber.mark_changed();
        subscriber
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, T> {
        Ref(self.tx.borrow())
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

    #[allow(clippy::missing_panics_doc, reason = "Never panics.")]
    pub fn modify<M, N>(&self, modify: M) -> N
    where
        M: FnOnce(&mut T) -> N,
        N: ModifiedStatus,
    {
        // Workaround for the missing extensibility in Tokio.
        // For N = bool this boils down to `self.modify(modify)`.
        // See also: <https://github.com/tokio-rs/tokio/pull/7157>
        let mut modified_status_holder = None;
        self.tx.send_if_modified(|value| {
            let modified_status = modify(value);
            let is_modified = modified_status.is_modified();
            modified_status_holder = Some(modified_status);
            is_modified
        });
        modified_status_holder
            .expect("has been set by invoking the modify closure in the locking scope")
    }

    pub fn set_modified(&self) {
        self.modify(|_| true);
    }
}

impl<T> Clone for Publisher<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
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
pub struct Observer<T> {
    tx: watch::Sender<T>,
}

impl<T> Observer<T> {
    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        Subscriber::new(self.tx.subscribe())
    }

    #[must_use]
    pub fn subscribe_changed(&self) -> Subscriber<T> {
        let mut subscriber = self.subscribe();
        subscriber.mark_changed();
        subscriber
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, T> {
        Ref(self.tx.borrow())
    }
}

impl<T> Clone for Observer<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Subscriber<T> {
    rx: watch::Receiver<T>,
}

impl<T> Subscriber<T> {
    const fn new(rx: watch::Receiver<T>) -> Self {
        Self { rx }
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, T> {
        Ref(self.rx.borrow())
    }

    #[must_use]
    pub fn read_ack(&mut self) -> Ref<'_, T> {
        Ref(self.rx.borrow_and_update())
    }

    pub fn mark_changed(&mut self) {
        self.rx.mark_changed();
    }

    pub async fn changed(&mut self) -> Result<(), OrphanedSubscriberError> {
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

// Implementing Clone manually is required because #[derive(Clone)] would
// require T to be Clone, too.
impl<T> Clone for Subscriber<T> {
    fn clone(&self) -> Self {
        let Self { rx } = self;
        Self { rx: rx.clone() }
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
    use crate::{ModifiedStatus, OrphanedSubscriberError};

    use super::{Observer, Publisher, Ref, Subscriber};

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

    impl<'r, T> crate::traits::Readable<'r, T, Ref<'r, T>> for Publisher<T> {
        fn read(&self) -> Ref<'_, T> {
            self.read()
        }
    }

    impl<'r, T> crate::traits::Readable<'r, T, Ref<'r, T>> for Observer<T> {
        fn read(&self) -> Ref<'_, T> {
            self.read()
        }
    }

    impl<'r, T> crate::traits::Readable<'r, T, Ref<'r, T>> for Subscriber<T> {
        fn read(&self) -> Ref<'_, T> {
            self.read()
        }
    }

    impl<'r, T> crate::traits::Subscribable<'r, T, Ref<'r, T>, Subscriber<T>> for Publisher<T> {
        fn subscribe(&self) -> Subscriber<T> {
            self.subscribe()
        }

        fn subscribe_changed(&self) -> Subscriber<T> {
            self.subscribe_changed()
        }
    }

    impl<'r, T> crate::traits::Subscribable<'r, T, Ref<'r, T>, Subscriber<T>> for Observer<T> {
        fn subscribe(&self) -> Subscriber<T> {
            self.subscribe()
        }

        fn subscribe_changed(&self) -> Subscriber<T> {
            self.subscribe_changed()
        }
    }

    impl<'r, T> crate::traits::Publisher<'r, T, Ref<'r, T>, Observer<T>, Subscriber<T>>
        for Publisher<T>
    {
        fn observe(&self) -> Observer<T> {
            self.observe()
        }

        fn has_subscribers(&self) -> bool {
            self.has_subscribers()
        }

        fn write(&self, new_value: T) {
            self.write(new_value);
        }

        fn replace(&self, new_value: T) -> T {
            self.replace(new_value)
        }

        fn modify<M, N>(&self, modify: M) -> N
        where
            M: FnOnce(&mut T) -> N,
            N: ModifiedStatus,
        {
            self.modify(modify)
        }

        fn set_modified(&self) {
            self.set_modified();
        }
    }

    impl<'r, T> crate::traits::Observer<'r, T, Ref<'r, T>, Subscriber<T>> for Observer<T> {}

    impl<'r, T> crate::traits::Subscriber<'r, T, Ref<'r, T>> for Subscriber<T> {
        fn read_ack(&mut self) -> Ref<'_, T> {
            self.read_ack()
        }
    }

    impl<T> crate::traits::ChangeListener for Subscriber<T> {
        fn mark_changed(&mut self) {
            self.mark_changed();
        }

        async fn changed(&mut self) -> Result<(), OrphanedSubscriberError> {
            self.changed().await
        }
    }
}
