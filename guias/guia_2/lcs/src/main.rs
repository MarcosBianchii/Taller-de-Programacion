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

type Grid = Mat<u32>;

// Returns the grid of longest common subsequence between string x and y.
fn calculate_grid(xchars: &[char], ychars: &[char]) -> Grid {
    let mut grid = Mat::new(0, xchars.len() + 1, ychars.len() + 1);

    for i in 1..=xchars.len() {
        for j in 1..=ychars.len() {
            if xchars[i - 1] == ychars[j - 1] {
                grid[i][j] = grid[i - 1][j - 1] + 1;
            } else {
                grid[i][j] = grid[i - 1][j].max(grid[i][j - 1]);
            }
        }
    }

    grid
}

/// Calcuates the longest common substring
/// between strings x and y using Myers' algorithm.
pub fn lcs(x: &str, y: &str) -> String {
    let xchars: Vec<char> = x.chars().collect();
    let ychars: Vec<char> = y.chars().collect();

    // Get lcs grid of strings.
    let grid = calculate_grid(&xchars, &ychars);

    let mut lcs = String::new();
    let mut i = x.len();
    let mut j = y.len();

    // Get longest common subsequence.
    while i > 0 && j > 0 {
        if xchars[i - 1] == ychars[j - 1] {
            // If the char is in both x and y.
            lcs.push(xchars[i - 1]);
            i -= 1;
            j -= 1;
        } else if grid[i - 1][j] > grid[i][j - 1] {
            // If the char is only in x.
            i -= 1;
        } else {
            // If the char is only in y.
            j -= 1;
        }
    }

    // Because we iterated backwards
    // we need to reverse the string.
    lcs.chars().rev().collect()
}

fn main() {
    let x: Vec<String> = read_file_lines("files/a.txt");
    let y: Vec<String> = read_file_lines("files/b.txt");

    for (x, y) in x.iter().zip(y.iter()) {
        print_diff(lcs(x, y), x, y, x.len(), y.len());
        println!();
    }
}
