// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

use std::panic;

use crate::{OrphanedSubscriberError, Subscriber};

#[allow(dead_code)]
pub(crate) async fn map_changed<S, T>(
    subscriber: &mut Subscriber<S>,
    mut map_fn: impl FnMut(&S) -> T,
) -> Result<T, OrphanedSubscriberError> {
    let next_changed = subscriber.read_changed().await?;
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| map_fn(&next_changed)));
    match result {
        Ok(next_item) => Ok(next_item),
        Err(panicked) => {
            // Drop the read-lock to avoid poisoning it.
            drop(next_changed);
            // Forward the panic to the caller.
            panic::resume_unwind(panicked);
            // Unreachable
        }
    }
}

#[allow(dead_code)]
pub(crate) async fn filter_map_changed<S, T>(
    subscriber: &mut Subscriber<S>,
    mut filter_map_fn: impl FnMut(&S) -> Option<T>,
) -> Result<T, OrphanedSubscriberError> {
    loop {
        let next_changed = subscriber.read_changed().await?;
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| filter_map_fn(&next_changed)));
        match result {
            Ok(Some(next_item)) => {
                return Ok(next_item);
            }
            Ok(None) => {
                continue;
            }
            Err(panicked) => {
                // Drop the read-lock to avoid poisoning it.
                drop(next_changed);
                // Forward the panic to the caller.
                panic::resume_unwind(panicked);
                // Unreachable
            }
        }
    }
}
