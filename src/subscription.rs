use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Weak},
};

use crate::{Observer, Publisher, Subscriber};

#[derive(Debug)]
pub enum SubscriptionState<V, E> {
    Subscribing,
    SubscribeFailed(E),
    Subscribed(V),
    Unsubscribing,
    UnsubscribeFailed(E),
    Unsubscribed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionHandle<K> {
    key: Arc<K>,
}

impl<K> SubscriptionHandle<K> {
    pub fn key(&self) -> &K {
        self.key.as_ref()
    }
}

#[derive(Debug)]
pub struct SubscriptionObserver<K, V, E> {
    handle: SubscriptionHandle<K>,
    observer: Observer<SubscriptionState<V, E>>,
}

impl<K, V, E> SubscriptionObserver<K, V, E> {
    pub fn handle(&self) -> &SubscriptionHandle<K> {
        &self.handle
    }

    pub fn subscribe(&self) -> Subscriber<SubscriptionState<V, E>> {
        self.observer.subscribe()
    }
}

pub struct ManagedSubscription<K, V, E> {
    key: Weak<K>,
    observer: Observer<SubscriptionState<V, E>>,
}

pub struct SubscriptionPublisher<K, V, E> {
    key: Weak<K>,
    publisher: Publisher<SubscriptionState<V, E>>,
}

pub struct SubscriptionRegistry<K, V, E> {
    subscriptions: HashMap<K, ManagedSubscription<K, V, E>>,
}

impl<K, V, E> Default for SubscriptionRegistry<K, V, E> {
    fn default() -> Self {
        Self {
            subscriptions: HashMap::default(),
        }
    }
}

impl<K, V, E> SubscriptionRegistry<K, V, E>
where
    K: Eq + Hash,
{
    #[must_use]
    pub fn register(
        &mut self,
        key: K,
    ) -> (
        Option<SubscriptionPublisher<K, V, E>>,
        SubscriptionObserver<K, V, E>,
    ) {
        let Self { subscriptions } = self;
        if let Some(subscription) = subscriptions.get(&key)
            && let Some(key) = subscription.key.upgrade()
        {
            // Publisher already exists.
            return (
                None,
                SubscriptionObserver {
                    handle: SubscriptionHandle { key },
                    observer: subscription.observer.clone(),
                },
            );
        }
        // Create and register new publisher.
        let key = Arc::new(key);
        let publisher = Publisher::new(SubscriptionState::Subscribing);
        let observer = publisher.observe();
        let publisher = SubscriptionPublisher {
            key: Arc::downgrade(&key),
            publisher: Publisher::new(SubscriptionState::Subscribing),
        };
        let observer = SubscriptionObserver {
            handle: SubscriptionHandle { key },
            observer,
        };
        (Some(publisher), observer)
    }

    #[must_use]
    pub fn unregister_publisher(
        &mut self,
        key: &K,
    ) -> Option<(K, Observer<SubscriptionState<V, E>>)> {
        let Self { subscriptions } = self;
        subscriptions
            .remove_entry(key)
            .map(|(key, ManagedSubscription { key: _, observer })| (key, observer))
    }
}
