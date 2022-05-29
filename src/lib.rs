//! discro
//!
//! Discrete observables for *Functional Reactive Programming* (FRP).

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]
#![cfg_attr(channel = "nightly", feature(doc_cfg))]

#[cfg(feature = "tokio")]
#[cfg_attr(channel = "nightly", doc(cfg(feature = "tokio")))]
mod tokio;

#[cfg(feature = "tokio")]
#[cfg_attr(channel = "nightly", doc(cfg(feature = "tokio")))]
pub use self::tokio::*;
