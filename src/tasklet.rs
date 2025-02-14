// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

//! Tasklets for processing observed values.

use std::{future::Future, panic};

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
/// The `on_changed_fn` closure is invoked on every changed value.
/// The shared value is read-locked during the invocation!
///
/// No `_async` variant of this function could be provided, because
/// holding locks across yield points is not permitted.
pub async fn observe_changes<T>(
    mut subscriber: Subscriber<T>,
    mut on_changed_fn: impl FnMut(&T) -> OnChanged,
) {
    while let Ok(next_changed_ref) = subscriber.read_changed().await {
        let result =
            panic::catch_unwind(panic::AssertUnwindSafe(|| on_changed_fn(&next_changed_ref)));
        // Drop the read-lock to avoid poisoning it.
        drop(next_changed_ref);
        match result {
            Ok(on_changed) => match on_changed {
                OnChanged::Continue => {
                    // Consumed.
                    continue;
                }
                OnChanged::Abort => {
                    // Aborted by the consumer.
                    break;
                }
            },
            Err(panicked) => {
                // Forward the panic to the caller.
                panic::resume_unwind(panicked);
            }
        }
        // Unreachable
    }
    // Publisher has disappeared.
}

/// Capture changes while observing a shared value.
///
/// The `capture_changed_value_fn` closure transforms a borrowed reference
/// of the observed value into an owned instance of the captured value.
/// Typically `Clone::clone` is used for this purpose if the
/// observed and captured types are identical. Returning `false` indicates
/// that the value has not changed semantically, even if it has been modified.
///
/// The `on_changed_value_fn` closure is invoked after `capture_changed_value_fn`
/// return `true`. No locks are held during an invocation. The returned
/// `OnChanged` enum determines whether to continue or abort listening
/// for subsequent changes.
pub async fn capture_changes<S, T>(
    mut subscriber: Subscriber<S>,
    initial_value: T,
    mut capture_changed_value_fn: impl FnMut(&mut T, &S) -> bool,
    mut on_changed_value_fn: impl FnMut(&T) -> OnChanged,
) {
    let mut value = initial_value;
    loop {
        {
            let Ok(next_changed_ref) = subscriber.read_changed().await else {
                // Publisher has disappeared.
                break;
            };
            match panic::catch_unwind(panic::AssertUnwindSafe(|| {
                capture_changed_value_fn(&mut value, &next_changed_ref)
            })) {
                Ok(true) => (),
                Ok(false) => {
                    // No new, changed value observed.
                    continue;
                }
                Err(panicked) => {
                    // Drop the read-lock to avoid poisoning it.
                    drop(next_changed_ref);
                    // Forward the panic to the caller.
                    panic::resume_unwind(panicked);
                    // Unreachable
                }
            }
        };
        // Handle the changed value after dropping the read-lock.
        match on_changed_value_fn(&value) {
            OnChanged::Continue => {
                // Consumed.
                continue;
            }
            OnChanged::Abort => {
                // Aborted by the consumer.
                return;
            }
        }
    }
    // Publisher has disappeared.
}

/// Capture changes asynchronously while observing a shared value.
///
/// Same as [`capture_changes()`] with the only difference that the
/// `on_changed_value_fn` closure returns a future with the result.
#[expect(clippy::manual_async_fn)] // Required to validate the trait bounds of the return type.
pub fn capture_changes_async<'a, S, T, F>(
    mut subscriber: Subscriber<S>,
    initial_value: T,
    mut capture_changed_value_fn: impl FnMut(&mut T, &S) -> bool + Send + 'a,
    mut on_changed_value_fn: impl FnMut(&T) -> F + Send + 'a,
) -> impl Future<Output = ()> + Send + 'a
where
    // `tokio::watch::Receiver<S>` is only `Send` if `S` is both `Send` and `Sync`
    S: Send + Sync + 'a,
    T: Send + 'a,
    F: Future<Output = OnChanged> + Send + 'a,
{
    async move {
        let capture_changed_value_fn = &mut capture_changed_value_fn;
        let on_changed_value_fn = &mut on_changed_value_fn;
        let mut value = initial_value;
        loop {
            {
                let Ok(next_changed_ref) = subscriber.read_changed().await else {
                    // Publisher has disappeared.
                    break;
                };
                match panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    capture_changed_value_fn(&mut value, &next_changed_ref)
                })) {
                    Ok(true) => (),
                    Ok(false) => {
                        // No new, changed value observed.
                        continue;
                    }
                    Err(panicked) => {
                        // Drop the read-lock to avoid poisoning it.
                        drop(next_changed_ref);
                        // Forward the panic to the caller.
                        panic::resume_unwind(panicked);
                        // Unreachable
                    }
                }
            };
            // Handle the changed value asynchronously after dropping the read-lock.
            match on_changed_value_fn(&value).await {
                OnChanged::Continue => {
                    // Consumed.
                    continue;
                }
                OnChanged::Abort => {
                    // Aborted by the consumer.
                    return;
                }
            }
        }
    }
}
