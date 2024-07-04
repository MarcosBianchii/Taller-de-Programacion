use std::ops::{Index, IndexMut};

/// A Matrix struct.
pub struct Mat<T> {
    data: Vec<T>,
    stride: usize,
}

impl<T> Index<usize> for Mat<T> {
    type Output = [T];
    fn index(&self, idx: usize) -> &Self::Output {
        let row = idx * self.stride;
        &self.data[row..row + self.stride]
    }
}

impl<T> IndexMut<usize> for Mat<T> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        let row = idx * self.stride;
        &mut self.data[row..row + self.stride]
    }
}

impl<T: Clone> Mat<T> {
    /// Creates a new matrix with shape n x m.
    pub fn new(val: T, n: usize, m: usize) -> Self {
        Self {
            data: vec![val; n * m],
            stride: m,
        }
    }
}
