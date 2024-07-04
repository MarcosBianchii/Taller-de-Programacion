use crate::{commit::Commit, pullrequest_controller};

/// Handles GET requests from client
pub fn handle_get(path: &str) -> String {
    if path.starts_with("/repos/") {
        handle_get_request(path)
    } else {
        "404 Not Found\r\n\r\n".to_string()
    }
}

/// Handles GET requests from client. There are three options:
/// * List pull requests: GET /repos/{repo}/pulls
/// * List commits en un pull request: GET /repos/{repo}/pulls/{pull_number}/commits
/// * Obtain a pull request: GET /repos/{repo}/pulls/{pulls_number}
fn handle_get_request(path: &str) -> String {
    if path.ends_with("/pulls") {
        pullrequest_controller::list_pull_requests(path)
    } else if path.ends_with("/commits") {
        list_commits_request(path)
    } else if path.contains("/pulls/") {
        obtain_pull_request(path)
    } else {
        "404 Not Found\r\n\r\n".to_string()
    }
}

// Return a HTTP response with a list of commits in a pull request.
fn list_commits_request(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() == 6 {
        let repo = parts[2];
        if !repo.is_empty() {
            if let Ok(pull_number) = parts[4].parse::<usize>() {
                let repo = repo.to_string() + ".git";
                let commits = match pullrequest_controller::get_commits(pull_number, &repo) {
                    Ok(commits) => commits,
                    Err(_) => return "500 Error at get commits\r\n\r\n".to_string(),
                };

                let mut fmt_commits = vec![];
                for hash in commits {
                    match Commit::new(&hash, &repo) {
                        Ok(commit) => fmt_commits.push(commit),
                        Err(_) => return "500 Error at get commits\r\n\r\n".to_string(),
                    }
                }

                match serde_json::to_string(&fmt_commits) {
                    Ok(json) => {
                        return format!("200 OK\r\nContent-Type: application/json\r\n\r\n{json}")
                    }
                    Err(_) => return "500 Error at get commits\r\n\r\n".to_string(),
                };
            }
        }
    }

    "404 Not Found\r\n\r\n".to_string()
}

/// Get number of pull request and call a fucntion to send response with
/// information about that pull request.
/// GET /repos/{repo}/pulls/{pull_number}
fn obtain_pull_request(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() == 5 {
        let repo = parts[2].to_string() + ".git";
        if !repo.is_empty() {
            if let Ok(_pull_number) = parts[4].parse::<usize>() {
                return pullrequest_controller::get_pull_request(
                    format!("{}/{}", repo, parts[3]).as_str(),
                    _pull_number,
                );
            }
        }
    }

    "404 Not Found\r\n\r\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test() {
        let commits = vec![
            "905e2e9bb7ef73eec7d227f30e504362d04a0ecb",
            "2d14ca89afd275574b078eccd3db35d591f1753f",
            "9acfa8a85e2d572f44ec7d9b024612c95c9262a2",
        ];

        let mut fmt_commits = vec![];
        for hash in commits {
            let commit = Commit::new(hash, "repo.git").unwrap();
            fmt_commits.push(commit);
        }

        println!("{:?}", fmt_commits);

        let json = serde_json::to_string(&fmt_commits).unwrap();
        println!("{}", json);
    }
}
