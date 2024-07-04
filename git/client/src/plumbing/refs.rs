use std::{fs, io};

// Validates that the given branch exists in refs/heads.
// Returns the hash of it's latest commit.
pub fn get_branch_ref(branch: &str) -> io::Result<String> {
    get_ref(branch, "heads")
}

/// Validates that the given remote exists in refs/remotes.
/// Returns the hash of it's latest commit.
pub fn get_remote_ref(remote: &str) -> io::Result<String> {
    get_ref(remote, "remotes")
}

/// Returns the hash of the commit that .git/refs/<subfolder>/<name> points to.
pub fn get_ref(branch: &str, subfolder: &str) -> io::Result<String> {
    // por ahora saqué: el map
    let path = format!(".git/refs/{}/{}", subfolder, branch);
    let s = fs::read_to_string(path);
    match s {
        Ok(s) => Ok(s.replace('\n', "")),
        Err(err) => {
            println!("{}", err);
            Err(err)
        }
    }
}

/// Returns a list with all the local branches
pub fn get_local_branches() -> io::Result<Vec<String>> {
    let path = ".git/refs/heads";
    let mut branches = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(branch) = path.file_name() {
            branches.push(branch.to_string_lossy().to_string());
        }
    }
    Ok(branches)
}
