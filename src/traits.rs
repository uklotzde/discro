// SPDX-License-Identifier: MPL-2.0

//! Generic traits for verifying that implementations are compliant.

#![allow(missing_docs)]

use std::ops::Deref;

use async_trait::async_trait;

use super::OrphanedSubscriberError;

pub(crate) trait Readable<'r, R>
where
    R: 'r,
{
    #[must_use]
    fn read(&'r self) -> R;
}

pub(crate) trait Publisher<'r, T, R, S>: Readable<'r, R>
where
    R: Deref<Target = T> + 'r,
    S: Subscriber<'r, T, R>,
{
    #[must_use]
    fn subscribe(&self) -> S;

    fn write(&self, new_value: impl Into<T>);

    fn modify<M>(&self, modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool;
}

/// Read a shared value.
pub(crate) trait Subscriber<'r, T, R>: Readable<'r, R>
where
    R: Deref<Target = T> + 'r,
{
    #[must_use]
    fn read_ack(&'r mut self) -> (R, bool);
}

#[async_trait]
pub(crate) trait ChangeListener {
    async fn changed(&mut self) -> Result<(), OrphanedSubscriberError>;
}
