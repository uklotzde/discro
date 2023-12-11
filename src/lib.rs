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
