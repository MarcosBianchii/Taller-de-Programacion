use utils::{
    get_ancestor_from_repo, get_branch_from_repo, io_err, object::object_db::get_object_from_repo,
    plumbing::commit::get_commits_until_from_repo,
};

use crate::__get_pull_requests;
use crate::body_parameters::BodyParameters;
use crate::merge::merge_pr;
use crate::put_handler::PutBodyParameters;
use std::fs::File;
use std::io::Write;
use std::{fs, io};

/// PUT /repos/{repo}/pulls/{pull_number}/merge
/// Merge a PullRequest
pub fn merge_pull_request(path: &str) -> String {
    let (repo, pull_number) = match crate::put_handler::parse_path(path) {
        Ok((repo, pull_number)) => (repo, pull_number),
        Err(e) => return e.to_string(),
    };

    let prs_path = format!("{repo}/pulls");
    let mut prs = match __get_pull_requests(&prs_path) {
        Err(e) => return e.to_string(),
        Ok(prs) => prs,
    };

    let mut pr = match prs.remove(&pull_number) {
        Some(pr) => pr,
        None => return "404 Pull request not found\r\n\r\n".to_string(),
    };

    if !pr.is_mergeable() {
        return "405 Pull request is not mergeable\r\n\r\n".to_string();
    }

    if pr.is_merged() {
        return "405 Pull request is already merged\r\n\r\n".to_string();
    }

    let hash = match merge_pr(&repo, pr.get_base(), pr.get_head()) {
        Ok(hash) => hash,
        Err(e) => {
            println!("{e:?}");
            return "405 Merge failed\r\n\r\n".to_string();
        }
    };

    pr.set_merged();
    if pr.save().is_err() {
        return "500 Error saving pull request\r\n\r\n".to_string();
    }

    let body = PutBodyParameters {
        sha: hash,
        merged: true,
        message: "Pull Request successfully merged".to_string(),
    };

    let json = match serde_json::to_string(&body) {
        Ok(json) => json,
        Err(_) => return "500 Error serializing body\r\n\r\n".to_string(),
    };

    format!("200 OK\r\nContent-Type: application/json\r\n\r\n{json}")
}

/// GET /repos/{repo}/pulls
/// Lists all PullRequests from a repository
pub fn list_pull_requests(path: &str) -> String {
    // validate path
    let _path = path.replace("/repos/", "");
    let _path: Vec<_> = _path.split('/').collect();
    let path = format!("{}.git/{}", _path[0], _path[1]);

    let mut pr_vec = vec![];
    let pull_requests = match __get_pull_requests(&path) {
        Err(e) => return e.to_string(),
        Ok(pull_requests) => pull_requests,
    };

    // turn pull requests into vector
    for pull_request in pull_requests.values() {
        pr_vec.push(pull_request)
    }

    // sort prs by id number
    pr_vec.sort_by_key(|a| a.get_id());
    let prs_serialized = match serde_json::to_string(&pr_vec) {
        Ok(serialized) => serialized,
        Err(_) => return "500 Error at serialize\r\n\r\n".to_string(),
    };

    format!("200 OK\r\nContent-Type: application/json\r\n\r\n{prs_serialized}")
}

/// GET repos/{repo}/pulls/{pull_number}/commits
/// Get all commits from a PullRequest
pub fn get_commits(pull_number: usize, repo: &str) -> io::Result<Vec<String>> {
    let path = format!("{repo}/pulls");

    let prs = __get_pull_requests(&path).map_err(|_| io_err!("Error at get pull requests"))?;

    let pr = match prs.get(&pull_number) {
        None => return Err(io_err!("No pull request exists with that id")),
        Some(pr) => pr,
    };

    let hash_base: String = match get_branch_from_repo(&pr.base, repo) {
        Err(_) => return Err(io_err!("Error getting base")),
        Ok(hash_base) => hash_base,
    };

    let hash_head = match get_branch_from_repo(&pr.head, repo) {
        Err(_) => return Err(io_err!("Error getting head")),
        Ok(hash_head) => hash_head,
    };

    // Get data from base and head commits.
    let (_, _, base_data) = match get_object_from_repo(&hash_base, repo) {
        Err(e) => {
            println!("{:?}", e);
            return Err(io_err!("Error getting base data"));
        }
        Ok(data) => data,
    };

    let (_, _, head_data) = match get_object_from_repo(&hash_head, repo) {
        Err(e) => {
            println!("{:?}", e);
            return Err(io_err!("Error getting head data"));
        }
        Ok(data) => data,
    };

    let ancestor = get_ancestor_from_repo(&base_data, &head_data, repo)?;
    let commits = get_commits_until_from_repo(&hash_head, &ancestor, repo)?;
    Ok(commits)
}

/// GET /repos/{repo}/pulls/{pull_number}
/// Get one PullRequest
pub fn get_pull_request(_path: &str, pull_id: usize) -> String {
    let pull_requests = match __get_pull_requests(_path) {
        Err(e) => return e.to_string(),
        Ok(pull_requests) => pull_requests,
    };

    let pull_request = match pull_requests.get(&pull_id) {
        Some(pull_request) => pull_request,
        None => return "404 No pull request exists with that id\r\n\r\n".to_string(),
    };

    println!("Pull request with id: {pull_id}\n{pull_request:?}");

    let pull_request_serialized = match serde_json::to_string(&pull_request) {
        Ok(serialized) => serialized,
        Err(_) => return "500 Error at serialize\r\n\r\n".to_string(),
    };

    format!("200 OK\r\nContent-Type: application/json\r\n\r\n{pull_request_serialized}")
}

/// POST /repos/{repo}/pulls
/// Create a PullRequest
pub fn create_pull_request(path: &str, body: String) -> String {
    if path.starts_with("/repos/") && path.ends_with("/pulls") {
        let repo = match crate::post_handler::validate_repo_path(path) {
            Err(e) => return e.to_string(),
            Ok(repo) => repo,
        };

        // deserialize body of post request
        let body: BodyParameters = match serde_json::from_str(&body) {
            Err(_) => return "400 Error at deserialize body\r\n\r\n".to_string(),
            Ok(body) => body,
        };

        // 1) open to check if dir pulls exists, if not create it
        let path_repo_pulls = format!("{repo}/pulls");
        if fs::create_dir_all(&path_repo_pulls).is_err() {
            return "500 Error at creating pulls folder\r\n\r\n".to_string();
        }

        // 2) create pull request
        let pull_request = match crate::post_handler::create_pr(body, path, &repo) {
            Err(e) => return e.to_string(),
            Ok(pull_request) => pull_request,
        };

        // 3) serialize pull request and saved it in pulls/{pull_id}.
        let path_to_saved_pull_request = format!("{}/{}", path_repo_pulls, pull_request.get_id());
        let mut file = match File::create(path_to_saved_pull_request) {
            Err(_) => return "500 Error at creating pull file\r\n\r\n".to_string(),
            Ok(file) => file,
        };

        // serialize pull request
        let pull_request_serialized = match serde_json::to_string(&pull_request) {
            Err(_) => return "500 Error at serialize pull request\r\n\r\n".to_string(),
            Ok(pull_request_serialized) => pull_request_serialized,
        };

        if file.write_all(pull_request_serialized.as_bytes()).is_err() {
            return "500 Error at write\r\n\r\n".to_string();
        }

        // response format:
        //HTTP-Version Status-Code Reason-Phrase CRLF
        //headers CRLF
        //message-body
        format!("200 OK\r\nContent-Type: application/json\r\n\r\n{pull_request_serialized}")
    } else {
        "404 Not Found\r\n\r\n".to_string()
    }
}
