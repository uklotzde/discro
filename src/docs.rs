// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

//! This module is only provided for creating the documentation.

#![allow(dead_code)]
#![allow(unreachable_pub)]
#![allow(clippy::unused_self)]

use std::{marker::PhantomData, ops::Deref};

use super::OrphanedSubscriberError;

/// A borrowed reference to the shared value.
///
/// Outstanding borrows hold a read lock.
#[derive(Debug)]
pub struct Ref<T> {
    phantom: PhantomData<T>,
}

impl<T> AsRef<T> for Ref<T> {
    fn as_ref(&self) -> &T {
        unimplemented!();
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unimplemented!();
    }
}

/// Read-only projection of the [`Publisher`].
///
/// Could only be used for reading the current value or for subscribing.
///
/// Could be cloned to obtain multiple read-only projections.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct ReadOnlyPublisher<T> {
    phantom: PhantomData<T>,
}

impl<T> ReadOnlyPublisher<T> {
    /// Check if the publisher has subscribers.
    ///
    /// Returns `Some(true)` if at least one subscriber is connected
    /// or `Some(false)` if the publisher is orphaned. Returns `None`
    /// if the number of subscribers cannot be determined, e.g. if
    /// more than a single instance of this type exists.
    ///
    /// After returning `Some(false)` once the result must never change
    /// unless new [`ReadOnlyPublisher`]s are cloned from the actual
    /// [`Publisher`]! This ensures that no race conditions could occur.
    #[must_use]
    pub fn has_subscribers(&self) -> Option<bool> {
        unimplemented!()
    }

    /// Create a new subscription that is connected to this publisher.
    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        unimplemented!()
    }

    /// Obtain a reference to the most recent value.
    ///
    /// Outstanding borrows hold a read lock.
    #[must_use]
    pub fn read(&self) -> Ref<T> {
        unimplemented!()
    }
}

/// Read/write a shared value and emit change notifications on write.
///
/// All write methods operate on a borrowed reference `&self` and therefore require
/// interior mutability. This allows to share the publisher across multiple threads
/// and/or tasks by wrapping into an `Arc` without requiring explicit synchronization.
///
/// Only a single instance is supported, i.e. `Clone` is probably not implemented.
#[allow(missing_debug_implementations)]
pub struct Publisher<T> {
    phantom: PhantomData<T>,
}

impl<T> Publisher<T> {
    /// Create a new publisher without subscribers.
    #[must_use]
    pub fn new(initial_value: T) -> Self {
        drop(initial_value);
        unimplemented!()
    }

    /// Create a new read-only publisher.
    #[must_use]
    pub fn clone_read_only(&self) -> ReadOnlyPublisher<T> {
        unimplemented!()
    }

    /// [`ReadOnlyPublisher::has_subscribers`]
    #[must_use]
    pub fn has_subscribers(&self) -> Option<bool> {
        unimplemented!()
    }

    /// [`ReadOnlyPublisher::subscribe`]
    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        unimplemented!()
    }

    /// [`ReadOnlyPublisher::read`]
    #[must_use]
    pub fn read(&self) -> Ref<T> {
        unimplemented!()
    }

    /// Overwrite the current value with a new value
    /// and emit a change notification.
    ///
    /// The change notification is emitted unconditionally, i.e.
    /// independent of both the current and the new value.
    pub fn write(&self, new_value: impl Into<T>) {
        drop(new_value);
        unimplemented!()
    }

    /// Replace and return the current value with a new value
    /// and emit a change notification.
    ///
    /// The change notification is emitted unconditionally, i.e.
    /// independent of both the current and the new value.
    ///
    /// If you don't need the previous value, use [`write`](Self::write) instead.
    #[must_use]
    pub fn replace(&self, new_value: impl Into<T>) -> T {
        drop(new_value);
        unimplemented!()
    }

    /// Modify the current value in-place and conditionally emit a
    /// change notification.
    ///
    /// The mutable borrow of the current value is protected by
    /// a write lock during the execution scope of the `modify`
    /// closure.
    ///
    /// The result of the invoked `modify` closure controls if
    /// a change notification is sent or not. This result is
    /// finally returned.
    ///
    /// Return `true` to notify subscribers about the change, i.e.
    /// if the value has been modified and the modification is
    /// observable by subscribers.
    ///
    /// Return `false` to suppress change notifications, i.e. if
    /// the value has either not been modified or if the modification
    /// is not observable by subscribers.
    pub fn modify<M>(&self, modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool,
    {
        drop(modify);
        unimplemented!()
    }
}

/// Read a shared value and receive change notifications asynchronously.
///
/// If the initial value is considered as changed or not depends on both
/// the implementation and how the `Subscriber` has been created. Use
/// [`mark_changed()`](Self::mark_changed) to explicitly mark the current
/// value as _changed_ or reset it to _unchanged_ by calling
/// [`read_ack()`](Self::read_ack).
#[allow(missing_debug_implementations)]
pub struct Subscriber<T> {
    phantom: PhantomData<T>,
}

impl<T> Subscriber<T> {
    /// Read the current value
    ///
    /// Outstanding borrows hold a read lock. Trying to read the value
    /// again while already holding a read lock might cause a deadlock!
    #[must_use]
    pub fn read(&self) -> Ref<T> {
        unimplemented!()
    }

    /// Read and acknowledge the current value
    ///
    /// Outstanding borrows hold a read lock. Trying to read the value
    /// again while already holding a read lock might cause a deadlock!
    ///
    /// The current value is marked as _seen_ and therefore considered
    /// as _unchanged_ from now on.
    #[must_use]
    pub fn read_ack(&mut self) -> Ref<T> {
        unimplemented!()
    }

    /// Mark the current value as _changed_, i.e. _unseen_.
    pub fn mark_changed(&mut self) {
        unimplemented!()
    }

    /// Receive change notifications for the shared value.
    ///
    /// Waits for a change notification, then marks the newest value as seen.
    ///
    /// After subscribing, the first call to this method returns immediately.
    /// The current value after subscribing is always considered as _changed_.
    /// This also applies when creating a new subscriber by cloning an existing
    /// subscriber!
    ///
    /// # Errors
    ///
    /// Returns `Err(OrphanedSubscriberError)` if the subscriber is disconnected from the publisher.
    #[allow(clippy::unused_async)]
    pub async fn changed(&mut self) -> Result<(), OrphanedSubscriberError> {
        unimplemented!()
    }

    /// Read and acknowledge the next, changed value.
    ///
    /// Needed for creating streams with _at-most-once_ semantics.
    #[allow(clippy::unused_async)]
    pub async fn read_ack_changed(&mut self) -> Result<Ref<T>, OrphanedSubscriberError> {
        unimplemented!()
    }

    /// Observe modifications as a stream of changed values.
    ///
    /// Returns a stream of changed values.
    ///
    /// The `next_item_fn` closure is invoked on a borrowed value while the lock is held.
    #[cfg(feature = "async-stream")]
    pub fn into_changed_stream<U>(
        self,
        mut next_item_fn: impl FnMut(&T) -> U + Send,
    ) -> impl futures::Stream<Item = U> + Send
    where
        T: Send + Sync + 'static,
        U: Send + 'static,
    {
        // Minimal, non-working dummy implementation to satisfy the compiler.
        async_stream::stream! {
            let next_ref = self.read();
            let next_item = next_item_fn(&next_ref);
            yield next_item;
        }
    }

    /// Observe modifications as a stream of changed values.
    ///
    /// Returns a stream of values, starting with the first changed value for which
    /// `next_item_fn` returns `Some`.
    ///
    /// The `next_item_fn` closure is invoked on a borrowed value while the lock is held.
    #[cfg(feature = "async-stream")]
    pub fn into_changed_stream_filtered<U>(
        self,
        mut next_item_fn: impl FnMut(&T) -> Option<U> + Send,
    ) -> impl futures::Stream<Item = U> + Send
    where
        T: Send + Sync + 'static,
        U: Send + 'static,
    {
        // Minimal, non-working dummy implementation to satisfy the compiler.
        async_stream::stream! {
            let next_ref = self.read();
            let Some(next_item) = next_item_fn(&next_ref) else {
                return;
            };
            yield next_item;
        }
    }
}
