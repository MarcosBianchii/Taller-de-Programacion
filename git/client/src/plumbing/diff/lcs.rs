use std::collections::VecDeque;

use super::mat::Mat;
use std::fmt;
use std::fmt::Display;

/// Represents the differences
/// between lines in two files.
#[derive(Debug)]
pub enum FileDiff {
    Same(String),
    Added(String),
    Removed(String),
}

impl Display for FileDiff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FileDiff::*;
        match self {
            Same(line) => write!(f, "  {line}"),
            Added(line) => write!(f, "+ {line}"),
            Removed(line) => write!(f, "- {line}"),
        }
    }
}

type Grid = Mat<u32>;
/// Calclates the longest common subsequence grid
/// between the given lines of strings x and y.
fn lcs(xlines: &[&str], ylines: &[&str]) -> Grid {
    let mut grid = Grid::new(0, xlines.len() + 1, ylines.len() + 1);

    for (i, xline) in xlines.iter().enumerate() {
        for (j, yline) in ylines.iter().enumerate() {
            if xline == yline {
                grid[i + 1][j + 1] = grid[i][j] + 1;
            } else {
                grid[i + 1][j + 1] = grid[i + 1][j].max(grid[i][j + 1]);
            }
        }
    }

    grid
}

/// Get's the differences between lines in strings x and y.
fn diff_lines(c: Grid, x: &[&str], y: &[&str], i: usize, j: usize, diffs: &mut VecDeque<FileDiff>) {
    use FileDiff::*;
    if i > 0 && j > 0 && x[i - 1] == y[j - 1] {
        // Both lines are equal.
        diff_lines(c, x, y, i - 1, j - 1, diffs);
        let line = x[i - 1];
        diffs.push_back(Same(line.to_string()));
    } else if j > 0 && (i == 0 || c[i][j - 1] >= c[i - 1][j]) {
        // Line was added.
        diff_lines(c, x, y, i, j - 1, diffs);
        let line = y[j - 1];
        diffs.push_back(Added(line.to_string()));
    } else if i > 0 && (j == 0 || c[i][j - 1] < c[i - 1][j]) {
        // Line was removed.
        diff_lines(c, x, y, i - 1, j, diffs);
        let line = x[i - 1];
        diffs.push_back(Removed(line.to_string()));
    }
}

/// Get's the differences between the lines of the given strings.
pub fn diff(x: &str, y: &str) -> VecDeque<FileDiff> {
    // Get lines from strings.
    let xlines = x.lines().collect::<Vec<_>>();
    let ylines = y.lines().collect::<Vec<_>>();

    // Calculate LCS grid.
    let grid = lcs(&xlines, &ylines);

    // Get differences.
    let mut diffs = VecDeque::new();
    diff_lines(
        grid,
        &xlines,
        &ylines,
        xlines.len(),
        ylines.len(),
        &mut diffs,
    );

    diffs
}

#[cfg(test)]
mod tests {
    use super::*;
    fn print_diff(diff: VecDeque<FileDiff>) {
        for diff in diff {
            println!("{diff}");
        }
    }

    #[test]
    fn add() {
        let x = r#"
void func1() {
    x += 1
}

void func2() {
    x += 2
}
"#;
        let y = r#"
void func1() {
    x += 1
}

void func1.5() {
    x += 1.5
}

void func2() {
    x += 2
}
"#;
        let z = r#"
void func1() {
    x += 1
}

void func1.5() {
    x += 1.7
}

void func2() {
    x += 2
}
"#;

        let difference = diff(x, y);
        print_diff(difference);
        println!();
        let difference = diff(x, z);
        print_diff(difference);
    }

    #[test]
    fn modify() {
        let x = r#"#include <stdio.h>

int main() {
    printf("Hello, world!\n");
    return 0;
}
"#;
        let y = r#"#include <stdio.h>

int main() {
    // Dios mio, un comentario.
    printf("pepito\n");
    return 0;
}"#;
        let diff = diff(x, y);
        println!("diff: {diff:?}");
        print_diff(diff);
    }
}
