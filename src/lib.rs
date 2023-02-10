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

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(rustdoc::broken_intra_doc_links)]

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

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "tokio")]
pub use self::tokio::*;

pub mod tasklet;
