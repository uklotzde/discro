//! discro
//!
//! Discrete observables for asynchronous *Functional Reactive Programming* (FRP).

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use thiserror::Error;

mod docs;

#[cfg(not(any(feature = "tokio")))]
pub use self::docs::*;

mod traits;

/// Indicates that the publisher has been dropped.
#[derive(Error, Debug)]
#[error("disconnected from publisher")]
pub struct OrphanedSubscriberError;

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "tokio")]
pub use self::tokio::*;
