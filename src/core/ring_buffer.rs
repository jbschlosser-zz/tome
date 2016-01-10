use std::cmp;
use std::iter::Chain;
use std::slice::{Iter, IterMut};

#[derive(Debug)]
pub struct RingBuffer<T> {
    data: Vec<T>,
    max_elements: Option<usize>,
    next_index: usize
}

impl<T> RingBuffer<T> {
    pub fn new(max_elements: Option<usize>) -> RingBuffer<T> {
        let data = match max_elements {
            Some(max) => Vec::with_capacity(max),
            None => Vec::new()
        };
        RingBuffer {data: data, max_elements: max_elements, next_index: 0}
    }
    pub fn len(&self) -> usize { self.data.len() }
    pub fn push(&mut self, element: T) {
        // Insert the element.
        if self.next_index >= self.len() {
            self.data.push(element);
        } else {
            self.data[self.next_index] = element;
        }
        
        // Move to the next location.
        self.next_index = match self.max_elements {
            Some(max) => (self.next_index + 1) % max,
            None => self.next_index + 1
        };
    }
    pub fn get<'a>(&'a self, index: usize) -> &'a T {
        self.iter(index).next().expect("Index outside of expected range")
    }
    pub fn get_mut<'a>(&'a mut self, index: usize) -> &'a mut T {
        self.iter_mut(index).next().expect("Index outside of expected range")
    }
    pub fn get_recent<'a>(&'a self, index: usize) -> &'a T {
        self.most_recent(index + 1).next()
            .expect("Index outside of expected range")
    }
    pub fn get_recent_mut<'a>(&'a mut self, index: usize) -> &'a mut T {
        self.most_recent_mut(index + 1).next()
            .expect("Index outside of expected range")
    }
    pub fn iter<'a>(&'a self, start_index: usize) ->
        Chain<Iter<'a, T>, Iter<'a, T>>
    {
        let (second, first) = self.data.split_at(self.next_index);
        let first_skip = cmp::min(start_index, first.len());
        let second_skip =
            if start_index > first.len() {
                cmp::min(start_index - first.len(), second.len())
            } else {0};
        (&first[first_skip..]).iter().chain((&second[second_skip..]).iter())
    }
    pub fn iter_mut<'a>(&'a mut self, start_index: usize) ->
        Chain<IterMut<'a, T>, IterMut<'a, T>>
    {
        let (second, first) = self.data.split_at_mut(self.next_index);
        let first_skip = cmp::min(start_index, first.len());
        let second_skip =
            if start_index > first.len() {
                cmp::min(start_index - first.len(), second.len())
            } else {0};
        (&mut first[first_skip..]).iter_mut().chain(
            (&mut second[second_skip..]).iter_mut())
    }
    pub fn most_recent<'a>(&'a self, num: usize) ->
        Chain<Iter<'a, T>, Iter<'a, T>>
    {
        let start_index =
            if num > self.len() {
                // Scrolled back too far- stop at the beginning.
                0
            } else {
                self.len() - num
            };
        self.iter(start_index)
    }
    pub fn most_recent_mut<'a>(&'a mut self, num: usize) ->
        Chain<IterMut<'a, T>, IterMut<'a, T>>
    {
        let start_index =
            if num > self.len() {
                // Scrolled back too far- stop at the beginning.
                0
            } else {
                self.len() - num
            };
        self.iter_mut(start_index)
    }
}
