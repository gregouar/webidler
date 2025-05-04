/// A helper class to keep track of whether a data struct should be resync with the client.
/// Based on the fact the data was accessed as mutable.
pub struct LazySyncer<T> {
    inner: T,
    need_to_sync: bool,
}

impl<T> LazySyncer<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            need_to_sync: true,
        }
    }

    /// Get mutable access to the data struct and tag it as dirty
    pub fn mutate(&mut self) -> &mut T {
        self.need_to_sync = true;
        &mut self.inner
    }

    pub fn read(&self) -> &T {
        &self.inner
    }

    pub fn need_to_sync(&self) -> bool {
        self.need_to_sync
    }

    pub fn reset_sync(&mut self) {
        self.need_to_sync = false;
    }
}
