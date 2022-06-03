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
pub struct Ref<T>(T);

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Read/write a shared value and emit change notifications on write.
///
/// Publishers are not aware of how many [`Subscriber`]s are connected
/// who are observing changes.
#[allow(missing_debug_implementations)]
pub struct Publisher<T> {
    phantom_data: PhantomData<T>,
}

impl<T> Publisher<T> {
    /// Create a new subscription that is connected to this publisher.
    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        unimplemented!()
    }

    /// Replace the current value and emit a change notification.
    ///
    /// The change notification is emitted unconditionally, i.e.
    /// independent of both the current and the new value.
    pub fn write(&self, new_value: impl Into<T>) {
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
    pub fn modify<M>(&self, modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool,
    {
        drop(modify);
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

/// Read a shared value and receive change notifications asynchronously.
#[allow(missing_debug_implementations)]
pub struct Subscriber<T> {
    phantom_data: PhantomData<T>,
}

impl<T> Subscriber<T> {
    /// Obtain a reference to the most recent value.
    ///
    /// Outstanding borrows hold a read lock.
    #[must_use]
    pub fn read(&self) -> Ref<T> {
        unimplemented!()
    }

    /// Obtain a reference to the most recent value and mark that
    /// value as seen by acknowledging it.
    ///
    /// Outstanding borrows hold a read lock.
    #[must_use]
    pub fn read_ack(&mut self) -> Ref<T> {
        unimplemented!()
    }

    /// Receive change notifications for the shared value.
    ///
    /// Waits for a change notification, then marks the newest value as seen.
    ///
    /// # Errors
    ///
    /// Returns `Err(OrphanedSubscriberError)` if the subscriber is disconnected form the publisher.
    pub async fn changed(&mut self) -> Result<(), OrphanedSubscriberError> {
        unimplemented!()
    }
}

/// Create a [`Publisher`] with an initial [`Subscriber`].
pub fn new_pubsub<T>(initial_value: T) -> (Publisher<T>, Subscriber<T>) {
    drop(initial_value);
    unimplemented!()
}
