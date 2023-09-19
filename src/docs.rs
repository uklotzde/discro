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
#[allow(missing_debug_implementations)]
pub struct Subscriber<T> {
    phantom: PhantomData<T>,
}

impl<T> Subscriber<T> {
    /// Obtain a reference to the most recent value.
    ///
    /// Outstanding borrows hold a read lock. Trying to read the value
    /// again while already holding a read lock might cause a deadlock!
    #[must_use]
    pub fn read(&self) -> Ref<T> {
        unimplemented!()
    }

    /// Obtain a reference to the most recent value and mark that
    /// value as seen by acknowledging it.
    ///
    /// Returns a tuple with the borrowed value and a *changed flag*
    /// that indicates if changes have been detected and acknowledged.
    ///
    /// Callers must be prepared to handle *false positive* results, i.e.
    /// if the *changed flag* returns `true` even though the shared value
    /// has not been modified. The accuracy of the *changed flag* depends
    /// on the underlying implementation.
    ///
    /// Outstanding borrows hold a read lock. Trying to read the value
    /// again while already holding a read lock might cause a deadlock!
    #[must_use]
    pub fn read_ack(&mut self) -> Ref<T> {
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

    /// Observe modifications as a stream of captured values.
    ///
    /// Returns a stream of captured values, starting with the current value.
    ///
    /// The `capture_fn` closure is invoked on a borrowed value while the lock is held.
    #[cfg(feature = "async-stream")]
    pub fn into_stream<U>(
        self,
        mut capture_fn: impl FnMut(&T) -> U,
    ) -> impl futures::Stream<Item = U> {
        // Minimal, non-working dummy implementation to satisfy the compiler.
        async_stream::stream! {
            let captured = capture_fn(self.read().as_ref());
            yield captured;
        }
    }

    /// Observe modifications as a stream of captured values.
    ///
    /// Returns a stream of captured values, starting with the current value or the
    /// first value for which `capture_fn` returns `Some(_)`.
    ///
    /// The `capture_fn` closure is invoked on a borrowed value while the lock is held.
    /// Returning `Some(value)` from the closure will emit `value` on the stream.
    /// Returning `None` will skip the value and wait for the next change notification.
    #[cfg(feature = "async-stream")]
    pub fn into_stream_or_skip<U>(
        self,
        mut capture_or_skip_fn: impl FnMut(&T) -> Option<U>,
    ) -> impl futures::Stream<Item = U> {
        // Minimal, non-working dummy implementation to satisfy the compiler.
        async_stream::stream! {
            let Some(captured) = capture_or_skip_fn(self.read().as_ref()) else {
                return;
            };
            yield captured;
        }
    }

    /// Observe modifications as a stream of captured values.
    ///
    /// Returns a stream of captured values, starting with the current value or the
    /// first value for which `capture_or_defer_fn` returns `Ok(_)`.
    ///
    /// The `capture_or_defer_fn` closure is invoked on a borrowed value while the
    /// lock is held. Returning `Ok(value)` from the closure will emit `value` on the stream.
    /// Returning `R(defer)` will skip the value and instead race the returned `defer`
    /// future against the next change notification.
    ///
    /// Use case: Implementation of various throttling patterns that are applied _before_
    /// actually capturing the value. Capturing the borrowed value might be a costly
    /// operation that should be avoided if the value would be dropped anyway.
    #[cfg(feature = "async-stream")]
    pub fn into_stream_or_defer<U, R>(
        self,
        mut capture_or_defer_fn: impl FnMut(&T) -> Result<U, R>,
    ) -> impl futures::Stream<Item = U>
    where
        R: std::future::Future<Output = ()>,
    {
        // Minimal, non-working dummy implementation to satisfy the compiler.
        async_stream::stream! {
            let Ok(captured) = capture_or_defer_fn(self.read().as_ref()) else {
                return;
            };
            yield captured;
        }
    }
}
