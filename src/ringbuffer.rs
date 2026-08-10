use std::ops::Index;

pub struct RingBuffer<T, const N: usize> {
    data: [Option<T>; N],
    head: usize,
    len: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub const fn new() -> Self {
        Self {
            data: const { [const { None }; N] },
            head: 0,
            len: 0,
        }
    }

    fn index_of(&self, index: usize) -> usize {
        (self.head + index) % N
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn truncate(&mut self, len: usize) {
        if self.len > len {
            self.len = len
        }
    }

    pub fn push_mut(&mut self, value: T) -> &mut T {
        if self.len < N {
            let index = self.index_of(self.len);
            self.len += 1;
            self.data[self.head] = Some(value);
            self.data[index].as_mut().unwrap()
        } else {
            let index = self.head;
            self.head += 1;
            self.data[self.head] = Some(value);
            self.data[index].as_mut().unwrap()
        }
    }
}

impl<T, const N: usize> Index<usize> for RingBuffer<T, N> {
    type Output = Option<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[self.index_of(index)]
    }
}
