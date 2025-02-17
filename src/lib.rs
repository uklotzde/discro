// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

//! Discrete observables for asynchronous *Functional Reactive Programming* (FRP).
//!
//! ## Crate Features
//!
//! One of the following (mutually exclusive) features must be enabled to select
//! a concrete implementation:
//!
//! - `tokio` implementation based on [`tokio::sync::watch`](https://docs.rs/tokio/latest/tokio/sync/watch/)

use thiserror::Error;

mod docs;

#[cfg(not(any(feature = "tokio")))]
pub use self::docs::*;

#[cfg(test)]
mod traits;

/// Return value of [`Publisher::modify`].
///
/// Allows to capture data from within the locking scope.
/// If the modification does not capture and return any data
/// then `bool` could be used.
pub trait ModifiedStatus {
    /// Indicates if the shared value has been modified.
    ///
    /// Returns `true` if the shared value has been modified
    /// by a publisher and subscribers need to be notified.
    /// Returns `false` if the shared value is unchanged.
    ///
    /// See also: `[Publisher::set_modified]`.
    fn is_modified(&self) -> bool;
}

impl ModifiedStatus for bool {
    fn is_modified(&self) -> bool {
        *self
    }
}

/// Indicates that the publisher has been dropped.
#[derive(Error, Debug)]
#[error("disconnected from publisher")]
pub struct OrphanedSubscriberError;

pub(crate) mod subscriber;

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "tokio")]
pub use self::tokio::*;

#[cfg(feature = "async-stream")]
mod async_stream;

#[cfg(feature = "async-stream")]
pub use self::async_stream::*;

pub mod tasklet;
