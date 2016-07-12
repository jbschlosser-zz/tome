pub struct Indexed<T> {
    index: usize,
    index_limit: Box<FnMut(&T) -> usize>,
    pub data: T
}

impl<T> Indexed<T> {
    pub fn new<C: FnMut(&T) -> usize + 'static>(data: T, index_limit: C) -> Indexed<T> {
        Indexed { index: 0, index_limit: Box::new(index_limit), data: data }
    }
    pub fn index(&self) -> usize { self.index }
    pub fn set_index(&mut self, index: usize) {
        let limit = (*self.index_limit)(&self.data);
        if index < limit {
            self.index = index;
        } else {
            self.index = limit;
        }
    }
    pub fn increment_index(&mut self, amount: usize) {
        let index = self.index;
        self.set_index(index + amount);
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
    pub fn set_limit<C: FnMut(&T) -> usize + 'static>(&mut self, index_limit: C) {
        self.index_limit = Box::new(index_limit);
        let index = self.index;
        self.set_index(index);
    }
}
