//! Shallow wrapper around `tokio::sync::watch` primitives with an
//! opionionated API comprising of more recognizable function names
//! and hiding of unneeded features.

#![allow(missing_docs)]

use std::ops::Deref;

use async_trait::async_trait;
use tokio::sync::watch;

use super::OrphanedSubscriberError;

#[derive(Debug)]
pub struct Ref<'r, T>(watch::Ref<'r, T>);

impl<'r, T> Deref for Ref<'r, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[derive(Debug)]
pub struct Publisher<T> {
    tx: watch::Sender<T>,
}

impl<T> Publisher<T> {
    #[must_use]
    pub fn subscribe(&self) -> Subscriber<T> {
        Subscriber {
            rx: self.tx.subscribe(),
        }
    }

    pub fn write(&self, new_value: impl Into<T>) {
        // Sender::send() would prematurely abort and fail if
        // no senders are connected and the current value would
        // not be replaced as expected. Therefore we have to use
        // Sender::send_modify() here!
        self.tx.send_modify(|value| *value = new_value.into());
    }

    pub fn modify<M>(&self, modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool,
    {
        self.tx.send_if_modified(modify)
    }

    #[must_use]
    pub fn read(&self) -> Ref<'_, T> {
        Ref(self.tx.borrow())
    }
}

impl<'r, T> crate::traits::Publisher<'r, T, Ref<'r, T>, Subscriber<T>> for Publisher<T> {
    fn subscribe(&self) -> Subscriber<T> {
        self.subscribe()
    }

    fn write(&self, new_value: impl Into<T>) {
        self.write(new_value);
    }

    fn modify<M>(&self, modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool,
    {
        self.modify(modify)
    }
}

impl<'r, T> crate::traits::Readable<'r, Ref<'r, T>> for Publisher<T> {
    fn read(&self) -> Ref<'_, T> {
        self.read()
    }
}

#[derive(Debug, Clone)]
pub struct Subscriber<T> {
    rx: watch::Receiver<T>,
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
}

impl<'r, T> crate::traits::Readable<'r, Ref<'r, T>> for Subscriber<T> {
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

pub fn new_pubsub<T>(initial_value: T) -> (Publisher<T>, Subscriber<T>) {
    let (tx, rx) = watch::channel(initial_value);
    (Publisher { tx }, Subscriber { rx })
}
