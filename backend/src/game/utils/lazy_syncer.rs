/// A helper class to keep track of whether a data struct should be resync with the client.
/// Based on the fact the data was accessed as mutable.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(
    transparent,
    bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    )
)]
pub struct LazySyncer<T: Clone> {
    inner: T,
    #[serde(skip)]
    need_to_sync: bool,
}

impl<T: Clone> LazySyncer<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            need_to_sync: true,
        }
    }

    pub fn unwrap(self) -> T {
        self.inner
    }

    /// Get mutable access to the data struct and tag it as dirty
    pub fn mutate(&mut self) -> &mut T {
        self.need_to_sync = true;
        &mut self.inner
    }

    pub fn read(&self) -> &T {
        &self.inner
    }

    pub fn sync(&mut self) -> Option<T> {
        match self.need_to_sync {
            true => {
                self.need_to_sync = false;
                Some(self.inner.clone())
            }
            false => None,
        }
    }

    pub fn need_to_sync(&self) -> bool {
        self.need_to_sync
    }
}

impl<T: Clone> From<T> for LazySyncer<T> {
    fn from(value: T) -> Self {
        LazySyncer::new(value)
    }
}
