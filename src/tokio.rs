//! Shallow wrapper around `tokio::sync::watch` primitives with an
//! opionionated API comprising of more recognizable function names
//! and hiding of unneeded features.

use std::ops::Deref;

use async_trait::async_trait;
use tokio::sync::watch;

use super::OrphanedError;

/// Create a [`Publisher`] with an initial [`Subscriber`].
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

impl<'r, T> crate::traits::Publisher<'r, T, Ref<'r, T>, Subscriber<T>> for Publisher<T> {
    fn subscribe(&self) -> Subscriber<T> {
        Subscriber {
            rx: self.tx.subscribe(),
        }
    }

    fn write(&self, new_value: impl Into<T>) {
        // Sender::send() would prematurely abort and fail if
        // no senders are connected and the current value would
        // not be replaced as expected. Therefore we have to use
        // Sender::send_modify() here!
        self.tx.send_modify(|value| *value = new_value.into());
    }

    #[cfg(feature = "tokio-send_if_modified")]
    fn modify<M>(&self, modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool,
    {
        self.tx.send_if_modified(modify)
    }

    #[cfg(not(feature = "tokio-send_if_modified"))]
    fn modify<M>(&self, _modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool,
    {
        todo!("requires tokio v0.19.0");
    }
}

impl<'r, T> crate::traits::Readable<'r, Ref<'r, T>> for Publisher<T> {
    fn read(&self) -> Ref<'_, T> {
        Ref(self.tx.borrow())
    }
}

/// Observe a shared value.
#[derive(Debug, Clone)]
pub struct Subscriber<T> {
    rx: watch::Receiver<T>,
}

impl<T> Subscriber<T> {
    #[allow(clippy::missing_errors_doc)]
    /// Non-boxing implementation of [`crate::traits::ChangeListener::changed()`]
    pub async fn changed(&mut self) -> Result<(), OrphanedError> {
        self.rx.changed().await.map_err(|_| OrphanedError)
    }
}

impl<'r, T> crate::traits::Readable<'r, Ref<'r, T>> for Subscriber<T> {
    fn read(&self) -> Ref<'_, T> {
        Ref(self.rx.borrow())
    }
}

impl<'r, T> crate::traits::Subscriber<'r, T, Ref<'r, T>> for Subscriber<T> {
    fn read_ack(&mut self) -> Ref<'_, T> {
        Ref(self.rx.borrow_and_update())
    }
}

#[async_trait]
impl<T> crate::traits::ChangeListener for Subscriber<T>
where
    T: Send + Sync,
{
    async fn changed(&mut self) -> Result<(), OrphanedError> {
        self.changed().await
    }
}

/// A borrowed reference to the shared value.
///
/// Outstanding borrows hold a read lock.
#[derive(Debug)]
pub struct Ref<'r, T>(watch::Ref<'r, T>);

impl<'r, T> Deref for Ref<'r, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
