//! A type-safe, concurrent container keyed by Rust types themselves.
//!
//! Useful for sharing context like databases, caches, or shared state.

use {
    std::{
        any::{Any, TypeId},
        collections::HashMap,
        sync::Arc,
    },
    tokio::sync::RwLock,
};

type DynamicMap = HashMap<TypeId, Box<dyn Any + Send + Sync>>;

/// [`TypeStore`] is a thread-safe, type-driven container for storing and retrieving
/// values by their concrete types at runtime.
///
/// Think of it like a dynamic, global registry keyed by types, suitable for sharing
/// context across async components.
#[derive(Debug, Clone, Default)]
pub struct TypeStore(Arc<RwLock<DynamicMap>>);

impl TypeStore {
    /// Retrieves a cloned value of type `T` from the store.
    pub async fn fetch<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        self.0
            .read()
            .await
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>().cloned())
    }

    /// Inserts a value of type `T` into the store.
    /// If a previous value existed, it will be returned.
    pub async fn insert<T>(&self, value: T) -> Option<T>
    where
        T: Send + Sync + 'static,
    {
        self.0
            .write()
            .await
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|old| old.downcast::<T>().ok().map(|boxed| *boxed))
    }

    /// Applies an update function to a mutable reference of the stored value of type `T`.
    pub async fn update<T>(&self, updater: impl FnOnce(&mut T))
    where
        T: 'static,
    {
        if let Some(item) = self
            .0
            .write()
            .await
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
        {
            updater(item);
        }
    }

    /// Returns `true` if a value of type `T` exists in the store.
    pub async fn contains<T>(&self) -> bool
    where
        T: 'static,
    {
        self.0.read().await.contains_key(&TypeId::of::<T>())
    }

    /// Removes the value of type `T` from the store, if present.
    pub async fn remove<T>(&self) -> Option<T>
    where
        T: 'static,
    {
        self.0
            .write()
            .await
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }
}
