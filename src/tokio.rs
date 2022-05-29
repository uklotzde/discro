//! Shallow wrapper around `tokio::sync::watch` primitives with an
//! opionionated API comprising of more recognizable function names
//! and hiding of unneeded features.

use thiserror::Error;
use tokio::sync::watch;

/// Create a [`Publisher`]/[`Subscriber`] pair.
pub fn new_pubsub<T>(initial_value: T) -> (Publisher<T>, Subscriber<T>) {
    let (tx, rx) = watch::channel(initial_value);
    (Publisher { tx }, Subscriber { rx })
}

/// Modify a shared value and emit change notifications.
///
/// The publisher is not aware of how many [`Subscriber`]s are
/// observing the changes.
#[derive(Debug)]
pub struct Publisher<T> {
    tx: watch::Sender<T>,
}

/// A borrowed reference to the shared value.
///
/// <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Ref.html>
pub type Ref<'a, T> = watch::Ref<'a, T>;

impl<T> Publisher<T> {
    /// Create a new [`Subscriber`] connected to this publisher.
    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        Subscriber {
            rx: self.tx.subscribe(),
        }
    }

    /// Replace the current value and emit a change notification.
    ///
    /// The change notification is emitted unconditionally, i.e.
    /// independent of both the current and the new value.
    pub fn update(&self, new_value: impl Into<T>) {
        // Sender::send() would prematurely abort and fail if
        // no senders are connected and the current value would
        // not be replaced as expected. Therefore we have to use
        // Sender::send_modify() here!
        self.tx.send_modify(|value| *value = new_value.into());
    }

    #[cfg(feature = "tokio-send_if_modified")]
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
    pub fn modify<M>(&self, modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool,
    {
        self.tx.send_if_modified(modify)
    }

    /// Obtain a reference to the most recently sent value.
    ///
    /// Outstanding borrows hold a read lock.
    ///
    /// <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Sender.html#method.borrow>
    #[must_use]
    pub fn peek(&self) -> Ref<'_, T> {
        self.tx.borrow()
    }
}

/// Observe a shared value.
#[derive(Debug, Clone)]
pub struct Subscriber<T> {
    rx: watch::Receiver<T>,
}

/// Indicates that the publisher has been dropped.
#[derive(Error, Debug)]
#[error(transparent)]
pub struct OrphanedError(#[from] watch::error::RecvError);

impl<T> Subscriber<T> {
    /// Receive change notifications.
    ///
    /// Waits for a change notification, then marks the newest value as seen.
    ///
    /// <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html#method.changed>
    ///
    /// # Errors
    ///
    /// Returns `Err(OrphanedError)` if the subscriber is disconnected form the publisher.
    pub async fn changed(&mut self) -> Result<(), OrphanedError> {
        self.rx.changed().await.map_err(OrphanedError)
    }

    /// Obtain a borrowed reference to the most recently sent value.
    ///
    /// Outstanding borrows hold a read lock.
    ///
    /// <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html#method.borrow>
    #[must_use]
    pub fn peek(&self) -> Ref<'_, T> {
        self.rx.borrow()
    }

    /// Obtain a borrowed reference to the most recently sent value and mark that value as seen.
    ///
    /// Outstanding borrows hold a read lock.
    ///
    /// <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html#method.borrow_and_update>
    #[must_use]
    pub fn take(&mut self) -> Ref<'_, T> {
        self.rx.borrow_and_update()
    }
}
