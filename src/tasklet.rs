// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

//! Tasklets for processing observed values.

use std::future::Future;

use crate::Subscriber;

/// Continuation after handling a change notification.
#[derive(Debug, Clone, Copy)]
pub enum OnChanged {
    /// Continue listening for changes
    Continue,

    /// Abort listening for changes
    Abort,
}

/// Observe a shared value.
///
/// The `on_changed` closure is invoked at least once when the tasklet
/// is started and then again after each change. The shared value is
/// locked during the invocation.
///
/// No `_async` variant of this function could be provided, because
/// holding locks across yield points is not permitted.
pub async fn observe_changes<T>(
    mut subscriber: Subscriber<T>,
    mut on_changed: impl FnMut(&T) -> OnChanged,
) {
    loop {
        let Ok(next_changed_ref) = subscriber.read_changed().await else {
            // Publisher has disappeared
            return;
        };
        match on_changed(&next_changed_ref) {
            OnChanged::Continue => (),
            OnChanged::Abort => {
                // Aborted by consumer
                return;
            }
        }
    }
}

/// Capture changes while observing a shared value.
///
/// The `capture` closure transforms a borrowed reference of the
/// observed value into an owned instance of the captured value.
/// Typically `Clone::clone` is used for this purpose if the
/// observed and captured types are identical.
///
/// The `has_changed` closure determines if the old/current captured
/// value (1st param) differs from the newly observed value (2nd param).
/// Typically `std::cmp::PartialEq::ne` is used for this purpose if the
/// observed and captured types are identical.
///
/// The `on_changed` closure is invoked after each change. No locks are held
/// during an invocation. The returned `OnChanged` enum determines whether
/// to continue or abort listening for subsequent changes.
pub async fn capture_changes<S, T>(
    mut subscriber: Subscriber<S>,
    initial_value: T,
    mut capture_changed_value: impl FnMut(&T, &S) -> Option<T>,
    mut on_changed_value: impl FnMut(&T) -> OnChanged,
) {
    let mut value = initial_value;
    loop {
        let Ok(next_changed_ref) = subscriber.read_changed().await else {
            // Publisher has disappeared.
            return;
        };
        let Some(new_value) = capture_changed_value(&value, &next_changed_ref) else {
            // No new, changed value.
            continue;
        };
        // Release the read-lock.
        drop(next_changed_ref);
        value = new_value;
        match on_changed_value(&value) {
            OnChanged::Continue => (),
            OnChanged::Abort => {
                // Aborted by consumer.
                return;
            }
        }
    }
}

/// Capture changes asynchronously while observing a shared value.
///
/// Same as [`capture_changes()`] with the only difference that
/// the `on_changed_value` closure returns a future with the result.
pub async fn capture_changes_async<S, T, F>(
    mut subscriber: Subscriber<S>,
    initial_value: T,
    mut capture_changed_value: impl FnMut(&T, &S) -> Option<T>,
    mut on_changed_value: impl FnMut(&T) -> F + Send + 'static,
) where
    S: Send + Sync + 'static,
    T: Send + Sync + 'static,
    F: Future<Output = OnChanged> + Send + 'static,
{
    let mut value = initial_value;
    loop {
        let Ok(next_changed_ref) = subscriber.read_changed().await else {
            // Publisher has disappeared.
            return;
        };
        let Some(new_value) = capture_changed_value(&value, &next_changed_ref) else {
            // No new, changed value.
            continue;
        };
        // Release the read-lock.
        drop(next_changed_ref);
        value = new_value;
        match on_changed_value(&value).await {
            OnChanged::Continue => (),
            OnChanged::Abort => {
                // Aborted by consumer.
                return;
            }
        }
    }
}
