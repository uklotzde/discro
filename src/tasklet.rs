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
/// locked during the invocation. It must return `true` to continue
/// and `false` to abort the task.
///
/// No `_async` variant of this function could be provided, because
/// holding locks across yield points is not permitted.
pub async fn observe_changes<T>(
    mut subscriber: Subscriber<T>,
    mut on_changed: impl FnMut(&T) -> OnChanged,
) {
    loop {
        match on_changed(&*subscriber.read_ack()) {
            OnChanged::Continue => (),
            OnChanged::Abort => {
                // Aborted by consumer
                return;
            }
        }
        if subscriber.changed().await.is_err() {
            // Publisher has disappeared
            return;
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
/// The `on_changed` closure is invoked at least once when the tasklet
/// is started and then again after each change. No locks are held
/// during an invocation. The returned `OnChanged` enum determines whether
/// to continue or abort listening for subsequent changes.
pub async fn capture_changes<S, T>(
    mut subscriber: Subscriber<S>,
    mut capture: impl FnMut(&S) -> T,
    mut has_changed: impl FnMut(&T, &S) -> bool,
    mut on_changed: impl FnMut(&T) -> OnChanged,
) {
    let mut value = capture(&*subscriber.read_ack());
    loop {
        match on_changed(&value) {
            OnChanged::Continue => (),
            OnChanged::Abort => {
                // Aborted by consumer
                return;
            }
        }
        loop {
            if subscriber.changed().await.is_err() {
                // Publisher has disappeared
                return;
            }
            let new_value = subscriber.read_ack();
            if has_changed(&value, &*new_value) {
                value = capture(&*new_value);
                // Exit inner loop for sending a notification
                break;
            }
        }
    }
}

/// Capture changes asynchronously while observing a shared value.
///
/// Same as [`capture_changes()`] with the only difference that
/// the `on_changed` closure returns a future with the result.
pub async fn capture_changes_async<S, T, F>(
    mut subscriber: Subscriber<S>,
    mut capture: impl FnMut(&S) -> T + Send + 'static,
    mut has_changed: impl FnMut(&T, &S) -> bool + Send + 'static,
    mut on_changed: impl FnMut(&T) -> F + Send + 'static,
) where
    S: Send + Sync + 'static,
    T: Send + Sync + 'static,
    F: Future<Output = OnChanged> + Send + 'static,
{
    let mut value = capture(&*subscriber.read_ack());
    loop {
        match on_changed(&value).await {
            OnChanged::Continue => (),
            OnChanged::Abort => {
                // Aborted by consumer
                return;
            }
        }
        loop {
            if subscriber.changed().await.is_err() {
                // Publisher has disappeared
                return;
            }
            let new_value = subscriber.read_ack();
            if has_changed(&value, &*new_value) {
                value = capture(&*new_value);
                // Exit inner loop for sending a notification
                break;
            }
        }
    }
}
