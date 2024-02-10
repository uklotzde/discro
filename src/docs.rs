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

    /// Subscribe to this publisher
    ///
    /// Listen for changes and read the shared value.
    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        unimplemented!()
    }

    /// Subscribe to this publisher
    ///
    /// Marks the subscriber as _changed_ to receive the current value immediately.
    #[must_use]
    pub fn subscribe_changed(&self) -> Subscriber<T> {
        unimplemented!()
    }

    /// Check if the publisher has subscribers.
    ///
    /// Returns `true` if at least one subscriber is connected
    /// or `false` otherwise.
    #[must_use]
    pub fn has_subscribers(&self) -> bool {
        unimplemented!()
    }

    /// Obtain a reference to the most recent value.
    ///
    /// Outstanding borrows hold a read lock.
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

    /// Mark the current value as _modified_, i.e. _changed_ for all subscribers.
    pub fn set_modified(&self) {
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
    pub async fn read_changed(&mut self) -> Result<Ref<T>, OrphanedSubscriberError> {
        unimplemented!()
    }

    /// Capture the next, changed value.
    ///
    /// The temporary reference is mapped to a custom type before returning.
    pub async fn map_changed<U>(
        &mut self,
        mut map_fn: impl FnMut(&T) -> U,
    ) -> Result<U, OrphanedSubscriberError> {
        self.read_changed().await.map(|next_ref| map_fn(&next_ref))
    }

    /// Capture the next, changed value conditionally.
    ///
    /// The temporary reference is filtered and mapped to a custom type before returning.
    pub async fn filter_map_changed<U>(
        &mut self,
        mut filter_map_fn: impl FnMut(&T) -> Option<U>,
    ) -> Result<U, OrphanedSubscriberError> {
        loop {
            let next_changed = self.read_changed().await?;
            if let Some(next_item) = filter_map_fn(&next_changed) {
                return Ok(next_item);
            }
        }
    }
}
