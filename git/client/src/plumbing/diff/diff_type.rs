#[derive(Debug, PartialEq)]
pub enum DiffType {
    Added,
    Modified(String),
    Unchanged,
    Removed,
}

#[derive(Debug)]
pub struct Diff {
    pub line: String,
    pub tag: DiffType,
}

impl Diff {
    /// Cretes a new Diff with a `Added` type.
    pub fn added(line: String) -> Self {
        Self {
            line,
            tag: DiffType::Added,
        }
    }

    /// Creates a new Diff with a 'Modified' type.
    pub fn modified(a: String, b: String) -> Self {
        Self {
            line: a,
            tag: DiffType::Modified(b),
        }
    }

    /// Creates a new Diff with a 'Unchanged' type.
    pub fn unchanged(line: String) -> Self {
        Self {
            line,
            tag: DiffType::Unchanged,
        }
    }

    /// Creates a new Diff with a `Removed` type.
    pub fn removed(line: String) -> Self {
        Self {
            line,
            tag: DiffType::Removed,
        }
    }
}
