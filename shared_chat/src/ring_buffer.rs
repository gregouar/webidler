use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buf: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.capacity == 0 {
            return;
        }

        if self.buf.len() == self.capacity {
            self.buf.pop_front();
        }
        self.buf.push_back(value);
    }

    pub fn extend<'a>(&mut self, iterator: impl Iterator<Item = T>)
    where
        T: 'a,
    {
        for iter in iterator {
            self.push(iter);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buf.iter()
    }

    pub fn iter_rev(&self) -> impl Iterator<Item = &T> {
        self.buf.iter().rev()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::RingBuffer;

    #[test]
    fn keeps_only_the_most_recent_values() {
        let mut buffer = RingBuffer::new(3);

        buffer.extend(1..=5);

        assert_eq!(buffer.iter().copied().collect::<Vec<_>>(), vec![3, 4, 5]);
        assert_eq!(
            buffer.iter_rev().copied().collect::<Vec<_>>(),
            vec![5, 4, 3]
        );
    }

    #[test]
    fn zero_capacity_buffer_never_stores_values() {
        let mut buffer = RingBuffer::new(0);

        buffer.push(1);

        assert!(buffer.is_empty());
    }
}
