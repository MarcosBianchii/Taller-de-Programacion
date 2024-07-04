use crate::pullrequest_controller;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct PutBodyParameters {
    pub sha: String,
    pub merged: bool,
    pub message: String,
}

// Returns the repo name and pull number from a path.
pub(crate) fn parse_path(path: &str) -> Result<(String, usize), &'static str> {
    let path = path
        .replace("/repos/", "")
        .replace("/pulls/", " ")
        .replace("/merge", "");

    let mut path = path.split_whitespace();
    match (path.next(), path.next()) {
        (Some(repo), Some(pull_number)) => {
            let pull_number = match pull_number.parse::<usize>() {
                Ok(pull_number) => pull_number,
                Err(_) => return Err("400 Invalid pull number\r\n\r\n"),
            };

            Ok((repo.to_string() + ".git", pull_number))
        }
        _ => Err("400 Invalid path\r\n\r\n"),
    }
}

// path: /repos/{repo}/pulls/{pull_number}/merge
pub fn handle_put(path: &str) -> String {
    pullrequest_controller::merge_pull_request(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_test() {
        let path = parse_path("/repos/<nombre>/pulls/10/merge");
        assert_eq!(path, Ok(("<nombre>.git".to_string(), 10)));
    }
}
