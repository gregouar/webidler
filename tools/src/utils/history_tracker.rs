pub struct HistoryTracker<T>
where
    T: Clone,
{
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,   // index of the current state
    len: usize,    // number of valid states
    cursor: usize, // offset from head (0 = current)
}

impl<T: Clone> HistoryTracker<T> {
    pub fn new(capacity: usize, initial: T) -> Self {
        assert!(capacity > 0);

        let mut buffer = vec![None; capacity];
        buffer[0] = Some(initial);

        Self {
            buffer,
            capacity,
            head: 0,
            len: 1,
            cursor: 0,
        }
    }

    fn index(&self, offset: isize) -> usize {
        ((self.head as isize + offset).rem_euclid(self.capacity as isize)) as usize
    }

    pub fn current(&self) -> &T {
        self.buffer[self.index(-(self.cursor as isize))]
            .as_ref()
            .expect("Current state must exist")
    }

    pub fn push(&mut self, value: T) {
        if self.cursor > 0 {
            // Move head to the current state
            self.head = self.index(-(self.cursor as isize));
            self.len -= self.cursor;
            self.cursor = 0;
        }

        self.head = self.index(1);
        self.buffer[self.head] = Some(value);

        if self.len < self.capacity {
            self.len += 1;
        }
    }

    pub fn can_undo(&self) -> bool {
        self.len > 1 && self.cursor < self.len - 1
    }

    pub fn can_redo(&self) -> bool {
        self.cursor > 0
    }

    pub fn undo(&mut self) -> Option<&T> {
        if self.can_undo() {
            self.cursor += 1;
            Some(self.current())
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&T> {
        if self.can_redo() {
            self.cursor -= 1;
            Some(self.current())
        } else {
            None
        }
    }
}
