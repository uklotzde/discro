// SPDX-FileCopyrightText: The discro authors
// SPDX-License-Identifier: MPL-2.0

use crate::{OrphanedSubscriberError, Subscriber};

/// Observe modifications as a stream of changed values.
///
/// Returns a stream of changed values.
///
/// The `next_item_fn` closure is invoked on a borrowed value while the lock is held.
pub fn subscriber_into_changed_stream<S, T>(
    mut subscriber: Subscriber<S>,
    mut next_item_fn: impl FnMut(&S) -> T + Send,
) -> impl futures_core::Stream<Item = T> + Send
where
    S: Send + Sync,
    T: Send,
{
    asynk_strim::stream_fn(|mut yielder| async move {
        let next_item_fn = &mut next_item_fn;
        #[expect(clippy::while_let_loop)]
        loop {
            match subscriber.map_changed(|next| next_item_fn(next)).await {
                Ok(next_item) => {
                    yielder.yield_item(next_item).await;
                }
                Err(OrphanedSubscriberError) => {
                    // Stream exhausted after publisher disappeared.
                    break;
                }
            }
        }
    })
}

/// Observe modifications as a stream of changed values.
///
/// Returns a stream of values, starting with the first changed value for which
/// `next_item_fn` returns `Some`.
///
/// The `next_item_fn` closure is invoked on a borrowed value while the lock is held.
pub fn subscriber_into_changed_stream_filtered<S, T>(
    mut subscriber: Subscriber<S>,
    mut next_item_fn: impl FnMut(&S) -> Option<T> + Send,
) -> impl futures_core::Stream<Item = T> + Send
where
    S: Send + Sync,
    T: Send,
{
    asynk_strim::stream_fn(|mut yielder| async move {
        let next_item_fn = &mut next_item_fn;
        #[expect(clippy::while_let_loop)]
        loop {
            match subscriber
                .filter_map_changed(|next| next_item_fn(next))
                .await
            {
                Ok(next_item) => {
                    yielder.yield_item(next_item).await;
                }
                Err(OrphanedSubscriberError) => {
                    // Stream exhausted after publisher disappeared.
                    break;
                }
            }
        }
    })
}
