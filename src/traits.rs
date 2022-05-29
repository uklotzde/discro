use std::ops::Deref;

use async_trait::async_trait;

use super::OrphanedError;

/// Inspect a shared value.
pub trait Readable<'r, R>
where
    R: 'r,
{
    /// Obtain a reference to the most recent value.
    ///
    /// Outstanding borrows hold a read lock.
    #[must_use]
    fn read(&'r self) -> R;
}

/// Read/write a shared value and emit change notifications on write.
///
/// Publishers are not aware of how many [`Subscriber`]s are connected
/// that are observing changes.
pub trait Publisher<'r, T, R, S>: Readable<'r, R>
where
    R: Deref<Target = T> + 'r,
    S: Subscriber<'r, T, R>,
{
    /// Create a new subscription that is connected to this publisher.
    #[must_use]
    fn subscribe(&self) -> S;

    /// Replace the current value and emit a change notification.
    ///
    /// The change notification is emitted unconditionally, i.e.
    /// independent of both the current and the new value.
    fn write(&self, new_value: impl Into<T>);

    /// Modify the current value in-place and conditionally emit a
    /// change notification.
    ///
    /// The mutable borrow of the current value is protected by
    /// a write lock during the execution scope of the `modify`
    /// closure.
    ///
    /// The result of the invoked `modify` closure controls if
    /// a change notification is sent or not. This result is
    /// passed through and then returned.
    fn modify<M>(&self, modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool;
}

/// Read a shared value.
pub trait Subscriber<'r, T, R>: Readable<'r, R>
where
    R: Deref<Target = T> + 'r,
{
    /// Obtain a borrowed reference to the most recently sent value and mark that
    /// value as seen by acknowledging it.
    ///
    /// Outstanding borrows hold a read lock.
    #[must_use]
    fn read_ack(&'r mut self) -> R;
}

/// Receive change notifications asynchronously.
///
/// This trait is only provided for documentation purposes and should
/// not actually be used. Instead subscribers are supposed to implement
/// a non-boxing version of it natively.
#[async_trait]
pub trait ChangeListener {
    /// Receive change notifications for the shared value.
    ///
    /// Waits for a change notification, then marks the newest value as seen.
    ///
    /// # Errors
    ///
    /// Returns `Err(OrphanedError)` if the subscriber is disconnected form the publisher.
    async fn changed(&mut self) -> Result<(), OrphanedError>;
}
