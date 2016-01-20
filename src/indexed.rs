use tome::{FormattedString, RingBuffer};

pub trait HasLength {
    fn len(&self) -> usize;
}

pub struct Indexed<T> {
    index: usize,
    pub data: T
}

impl<T: HasLength> Indexed<T> {
    pub fn new(data: T) -> Indexed<T> {
        Indexed { index: 0, data: data }
    }
    pub fn index(&self) -> usize { self.index }
    pub fn increment_index(&mut self, amount: usize) {
        if self.index + amount < self.data.len() {
            self.index += amount;
        } else {
            self.index = self.data.len() - 1;
        }
    }
    pub fn decrement_index(&mut self, amount: usize) {
        if amount > self.index {
            self.index = 0;
        } else {
            self.index -= amount;
        }
    }
    pub fn reset_index(&mut self) {
        self.index = 0;
    }
}

impl<T> HasLength for RingBuffer<T> {
    fn len(&self) -> usize { self.len() }
}

impl HasLength for FormattedString {
    fn len(&self) -> usize { self.len() }
}
