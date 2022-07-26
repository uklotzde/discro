// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

//! Tasklets for processing observed values.

use crate::Subscriber;

/// Observe a shared value.
///
/// The `on_changed` closure is invoked at least once when the tasklet
/// is started and then again after each change. The shared value is
/// locked during the invocation. It must return `true` to continue
/// and `false` to abort the task.
pub async fn observe_changes<T>(
    mut subscriber: Subscriber<T>,
    mut on_changed: impl FnMut(&T) -> bool,
) {
    loop {
        if !on_changed(&*subscriber.read_ack()) {
            // Aborted by consumer
            return;
        }
        if subscriber.changed().await.is_err() {
            // Publisher has disappeared
            return;
        }
    }
}

/// Capture changes by observing a shared value.
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
/// during an invocation. It must return `true` to continue and `false`
/// to abort the task.
pub async fn capture_changes<S, T>(
    mut subscriber: Subscriber<S>,
    mut capture: impl FnMut(&S) -> T,
    mut has_changed: impl FnMut(&T, &S) -> bool,
    mut on_changed: impl FnMut(&T) -> bool,
) {
    let mut value = capture(&*subscriber.read_ack());
    loop {
        if !on_changed(&value) {
            // Aborted by consumer
            return;
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
