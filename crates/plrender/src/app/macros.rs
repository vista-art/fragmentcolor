/// Handy macro that implements the app::Container trait for the given type.
///
/// The type must contain a `container` field and implement the Default
/// trait for this macro to work.
macro_rules! implements_container {
    ($type:ty, <$key:ty, $value:ty>) => {
        impl crate::app::container::Container<$key, $value> for $type {
            fn new() -> Self {
                Self {
                    container: std::collections::HashMap::new(),
                    ..Default::default()
                }
            }

            fn get(&self, id: $key) -> std::option::Option<std::sync::RwLockReadGuard<'_, $value>> {
                let value = self.container.get(id)?;
                let value = value.read().expect("Failed to acquire read lock");
                Some(value)
            }

            fn get_mut(
                &mut self,
                id: $key,
            ) -> std::option::Option<std::sync::RwLockWriteGuard<'_, $value>> {
                let value = self.container.get_mut(id)?;
                let value = value.write().expect("Failed to acquire write lock");
                Some(value)
            }

            fn insert(&mut self, id: $key, value: std::sync::Arc<std::sync::RwLock<$value>>) {
                self.container.insert(*id, value);
            }

            fn remove(
                &mut self,
                id: $key,
            ) -> std::option::Option<std::sync::Arc<std::sync::RwLock<$value>>> {
                self.container.remove(&id)
            }

            fn len(&self) -> usize {
                self.container.len()
            }
        }
    };
}

pub(crate) use implements_container;
