//! discro
//!
//! Discrete observables for *Functional Reactive Programming* (FRP).

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
#![cfg_attr(channel = "nightly", feature(doc_cfg))]

use thiserror::Error;

/// Generic traits
pub mod traits;

/// Re-export all traits in their unnameable form for convenience.
///
/// The async trait [`crate::traits::ChangeListener`] is not re-exported
/// as it is only provided for documentation purposes.
pub mod prelude {
    #[allow(unreachable_pub)]
    pub use crate::traits::{Readable as _, Publisher as _, Subscriber as _};
}

/// Indicates that the publisher has been dropped.
#[derive(Error, Debug)]
#[error("disconnected from publisher")]
pub struct OrphanedError;

#[cfg(feature = "tokio")]
#[cfg_attr(channel = "nightly", doc(cfg(feature = "tokio")))]
mod tokio;

#[cfg(feature = "tokio")]
#[cfg_attr(channel = "nightly", doc(cfg(feature = "tokio")))]
pub use self::tokio::*;
