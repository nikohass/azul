use async_mutex::Mutex;
use std::sync::Arc;

pub struct SharedState<T>(Arc<Mutex<T>>)
where
    T: Send + Sync;

impl<T> SharedState<T>
where
    T: Send + Sync,
{
    pub fn new(inner: T) -> Self {
        SharedState(Arc::new(Mutex::new(inner)))
    }

    pub async fn lock(&self) -> async_mutex::MutexGuard<'_, T> {
        self.0.lock().await
    }

    pub async fn replace(&self, new: T) {
        *self.0.lock().await = new;
    }
}

impl<T> Clone for SharedState<T>
where
    T: Send + Sync,
{
    fn clone(&self) -> Self {
        SharedState(self.0.clone())
    }
}

impl<T> Default for SharedState<T>
where
    T: Send + Sync + Default,
{
    fn default() -> Self {
        SharedState(Arc::new(Mutex::new(T::default())))
    }
}
