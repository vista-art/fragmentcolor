use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait Container<K, V> {
    fn new() -> Self;
    fn get(&self, id: K) -> Option<RwLockReadGuard<'_, V>>;
    fn get_mut(&mut self, id: K) -> Option<RwLockWriteGuard<'_, V>>;
    fn insert(&mut self, id: K, window: Arc<RwLock<V>>);
    fn remove(&mut self, id: K) -> Option<Arc<RwLock<V>>>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
