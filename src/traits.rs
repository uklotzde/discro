// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

//! Generic traits for verifying that implementations are compliant.

#![allow(missing_docs)]

use std::ops::Deref;

use async_trait::async_trait;

use super::OrphanedSubscriberError;

pub(crate) trait Ref<T>: AsRef<T> + Deref<Target = T> {}

pub(crate) trait Readable<'r, T, R>
where
    R: Ref<T> + 'r,
{
    #[must_use]
    fn read(&'r self) -> R;
}

pub(crate) trait ReadOnlyPublisher<'r, T, R, S>: Readable<'r, T, R>
where
    R: Ref<T> + 'r,
    S: Subscriber<'r, T, R>,
{
    #[must_use]
    fn has_subscribers(&self) -> bool;

    #[must_use]
    fn subscribe(&self) -> S;
}

pub(crate) trait Publisher<'r, T, R, S, P>: ReadOnlyPublisher<'r, T, R, S>
where
    R: Ref<T> + 'r,
    P: ReadOnlyPublisher<'r, T, R, S>,
    S: Subscriber<'r, T, R>,
{
    fn clone_read_only(&self) -> P;

    fn write(&self, new_value: impl Into<T>);

    #[must_use]
    fn replace(&self, new_value: impl Into<T>) -> T;

    fn modify<M>(&self, modify: M) -> bool
    where
        M: FnOnce(&mut T) -> bool;
}

pub(crate) trait Subscriber<'r, T, R>: Readable<'r, T, R>
where
    R: Ref<T> + 'r,
{
    #[must_use]
    fn read_ack(&'r mut self) -> R;
}

#[async_trait]
pub(crate) trait ChangeListener {
    async fn changed(&mut self) -> Result<(), OrphanedSubscriberError>;
}
