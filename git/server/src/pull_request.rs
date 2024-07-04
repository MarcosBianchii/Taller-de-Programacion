//! Structure that models a pull request
use std::fs::File;
use std::io;
use std::io::Write;

use serde::Deserialize;
use serde::Serialize;
/// When a pull request is created the serverhttp will
/// create a merge commit to test whether the pull request can be
/// merged into the base branch.
/// you can review the status of the test commit using the mergeable key, if mergeable
/// is true, then merge_commit_sha will be the commit of the merge test.
#[derive(Deserialize, Serialize, Debug)]
pub struct PullRequest {
    pub id: usize,
    pub head: String,
    pub base: String,
    pub repo: String,
    pub state: String,
    //pub commits: String,
    //pub additions: String,
    //pub deletions: String,
    //pub changed_files: String,
    pub merged: bool,
    pub mergeable: bool,
    //pub mergeable: bool,
    pub title: String,
    pub html_url: String,
    pub created_at: String,
    //pub closed_at: String,
    //pub merged_at: String,
    //pub merge_commit_sha: String,
}

impl PullRequest {
    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn is_mergeable(&self) -> bool {
        self.mergeable
    }

    pub fn is_merged(&self) -> bool {
        self.merged
    }

    pub fn get_base(&self) -> &str {
        &self.base
    }

    pub fn get_head(&self) -> &str {
        &self.head
    }

    pub fn set_merged(&mut self) {
        self.merged = true;
    }

    pub fn save(self) -> io::Result<()> {
        let repo = &self.repo;
        let pull_number = &self.id;
        let path = format!("{repo}/pulls/{pull_number}");

        let json = serde_json::to_string(&self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())
    }
}
