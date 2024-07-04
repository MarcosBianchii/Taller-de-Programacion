use super::diff_type::Diff;
use crate::plumbing::ls_tree::parse_ls_tree_entry;
use std::collections::HashMap;

/// Returns a vector of the differences
/// between the given trees.
///
/// # Note
///
/// Later utilization of this vector
/// assumes `Remove` diffs come before `Add` diffs.
pub fn diff_tree(a: &str, b: &str) -> impl Iterator<Item = Diff> {
    let mut diffs = vec![];

    // Create a HashMap of the first tree.
    let mut file1 = HashMap::new();
    for line in a.lines().filter(|line| !line.is_empty()) {
        let (mode, otype, hash, name) = parse_ls_tree_entry(line);
        file1.insert(name, (mode, otype, hash));
    }

    // Create a Hashmap of the second tree.
    let mut file2 = HashMap::new();
    for line in b.lines().filter(|line| !line.is_empty()) {
        let (mode, otype, hash, name) = parse_ls_tree_entry(line);
        file2.insert(name, (mode, otype, hash));
    }

    // Compare file1 to file2.
    for (name, data) in file1 {
        let (mode, otype, hash) = data;
        let line = format!("{mode} {otype} {hash} {name}\0");
        match file2.remove(&name) {
            None => {
                let diff = Diff::removed(line);
                diffs.push(diff);
            }
            Some(data) => {
                let (mode2, otype2, hash2) = data;
                // If the mode or object type changed
                // this means the file was renamed or
                // changed from a file to a directory
                // or viceversa.
                if mode != mode2 || otype != otype2 {
                    // Remove the old entry.
                    let diff = Diff::removed(line);
                    diffs.push(diff);

                    // Add the new entry.
                    let line = format!("{mode2} {otype2} {hash2} {name}\0");
                    let diff = Diff::added(line);
                    diffs.push(diff);
                } else if hash != hash2 {
                    // If only the hash changed this means
                    // the file content was modified or in
                    // the case of a directory it now
                    // contains different files.
                    let other = format!("{mode2} {otype2} {hash2} {name}\0");
                    let diff = Diff::modified(line, other);
                    diffs.push(diff);
                } else {
                    diffs.push(Diff::unchanged(line));
                }
            }
        }
    }

    // The entries left in file2
    // are all Diff::Add type.
    for (name, data) in file2 {
        let (mode, otype, hash) = data;
        let line = format!("{mode} {otype} {hash} {name}\0");
        let diff = Diff::added(line);
        diffs.push(diff);
    }

    diffs.into_iter()
}

/*
040000 tree e6f3d7bb6df0b70b77be69c602b9d4bdcc5e0ef4    .github
100644 blob ce6a0f315497cc7b7f6b8864617c10e89e59275f    .gitignore
100644 blob e8ffadba264e83be620b1c7b19a3b20bb429b4bc    notas_implementacion.txt
040000 tree 8695458f378758b6a8ccf7af74e395b83e60608a    server

040000 tree e6f3d7bb6df0b70b77be69c602b9d4bdcc5e0ef4    .github
100644 blob ce6a0f315497cc7b7f6b8864617c10e89e59275f    .gitignore
100644 blob b0490c9675eac72a51abae693878d87ebae4dc23    Cargo.toml
100644 blob ea5715a17138570b7c336acd9f91489853dcaca5    README.md
040000 tree f79792f6aa8f0ba9e72084fef38687cb5436db42    client
100644 blob e8ffadba264e83be620b1c7b19a3b20bb429b4bc    notas_implementacion.txt

040000 tree e6f3d7bb6df0b70b77be69c602b9d4bdcc5e0ef4    .github
100644 blob ce6a0f315497cc7b7f6b8864617c10e89e59275f    .gitignore
100644 blob e8ffadba264e83be620b1c7b19a3b20bb429b4bc    notas_implementacion.txt
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_remove() {
        let a = r#"
040000 tree e6f3d7bb6df0b70b77be69c602b9d4bdcc5e0ef4    .github
100644 blob ce6a0f315497cc7b7f6b8864617c10e89e59275f    .gitignore
100644 blob e8ffadba264e83be620b1c7b19a3b20bb429b4bc    notas_implementacion.txt
040000 tree 8695458f378758b6a8ccf7af74e395b83e60608a    server
"#;
        let b = r#"
040000 tree e6f3d7bb6df0b70b77be69c602b9d4bdcc5e0ef4    .github
100644 blob ce6a0f315497cc7b7f6b8864617c10e89e59275f    .gitignore
100644 blob b0490c9675eac72a51abae693878d87ebae4dc23    Cargo.toml
100644 blob ea5715a17138570b7c336acd9f91489853dcaca5    README.md
040000 tree f79792f6aa8f0ba9e72084fef38687cb5436db42    client
100644 blob e8ffadba264e83be620b1c7b19a3b20bb429b4bc    notas_implementacion.txt
"#;
        let diff: Vec<_> = diff_tree(a, b).collect();
        println!("{diff:#?}");
    }

    #[test]
    fn modified() {
        let a = r#"
040000 tree e6f3d7bb6df0b70b77be69c602b9d4bdcc5e0ef4    .github
100644 blob ce6a0f315497cc7b7f6b8864617c10e89e59275f    .gitignore
100644 blob e8ffadba264e83be620b1c7b19a3b20bb429b4bc    notas_implementacion.txt
"#;
        let b = r#"
040000 tree e6f3d7bb6df0b70b77be69c602b9d4bdcc5e0ef4    .github
100644 blob f315a098f907ee098acc7890987cafe7890d890e    .gitignore
100644 blob ea5715a17138570b7c336acd9f91489853dcaca5    README.md
"#;
        let diff: Vec<_> = diff_tree(a, b).collect();
        println!("{diff:#?}");
    }

    #[test]
    #[allow(unused_variables)]
    fn multi_diff() {
        let a = r#"
040000 tree e6f3d7bb6df0b70b77be69c602b9d4bdcc5e0ef4    .github
100644 blob ce6a0f315497cc7b7f6b8864617c10e89e59275f    .gitignore
100644 blob 1283750918273019827630498160598610298374    notas_implementacion.txt
040000 tree 8695458f378758b6a8ccf7af74e395b83e60608a    server
"#;
        let b = r#"
040000 tree e6f3d7bb6df0b70b77be69c602b9d4bdcc5e0ef4    .github
100644 blob ce6a0f315497cc7b7f6b8864617c10e89e59275f    .gitignore
100644 blob 7890483720398470598602986470189702984757    Cargo.toml
040000 tree 7581920398460985760129837409856701982734    tests
100644 blob e8ffadba264e83be620b1c7b19a3b20bb429b4bc    notas_implementacion.txt
"#;
        let ancestor = r#"
040000 tree e6f3d7bb6df0b70b77be69c602b9d4bdcc5e0ef4    .github
100644 blob ce6a0f315497cc7b7f6b8864617c10e89e59275f    .gitignore
100644 blob b0490c9675eac72a51abae693878d87ebae4dc23    Cargo.toml
100644 blob ea5715a17138570b7c336acd9f91489853dcaca5    README.md
040000 tree f79792f6aa8f0ba9e72084fef38687cb5436db42    client
100644 blob e8ffadba264e83be620b1c7b19a3b20bb429b4bc    notas_implementacion.txt
"#;
        let diff1: Vec<_> = diff_tree(ancestor, a).collect();
        let diff2: Vec<_> = diff_tree(ancestor, b).collect();
        println!("\n\n{diff1:#?}");
        println!("\n\n{diff2:#?}");
    }
}
