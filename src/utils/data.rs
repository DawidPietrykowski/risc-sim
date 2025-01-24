pub struct CircularBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
    pub size: usize,
}

impl<T> CircularBuffer<T>
where
    T: Default,
    T: Clone,
{
    pub fn new(size: usize) -> Self {
        CircularBuffer {
            buffer: vec![Default::default(); size],
            head: 0,
            tail: 0,
            size,
        }
    }

    #[allow(unused)]
    pub fn push(&mut self, item: T) {
        self.buffer[self.head] = item;
        self.head = (self.head + 1) % self.size;
        if self.head == self.tail {
            self.tail = (self.tail + 1) % self.size;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.tail != self.head {
            let item = self.buffer[self.tail].clone();
            self.tail = (self.tail + 1) % self.size;
            Some(item)
        } else {
            None
        }
    }
}
