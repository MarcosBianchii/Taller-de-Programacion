use crate::body_parameters::BodyParameters;
use crate::merge_test::validate_merge;
use crate::pull_request::PullRequest;
use crate::{__get_pull_requests, pullrequest_controller};
use chrono::DateTime;
use chrono::Utc;
use utils::get_branch_from_repo;

/// Returns the id of the next pull request
fn new_pull_request_id(repo_name: &str) -> Result<usize, &'static str> {
    // Solucion provisoria para pruebas con mas de un pr.
    let path_to_pulls = format!("{repo_name}/pulls");
    let _path = std::path::Path::new(&path_to_pulls);
    println!("El path antes de checkear existencia: {_path:?}");
    if !_path.exists() {
        println!("Entro otra vez a 0");
        return Ok(0);
    }

    let pull_requests = __get_pull_requests(&path_to_pulls)?;
    Ok(pull_requests.len())
}

// Tests whether a merge is possible or not.
fn merge_test(base: &str, head: &str, repo: &str) -> Result<bool, &'static str> {
    let hash_base: String = match get_branch_from_repo(base, repo) {
        Err(_) => return Err("400 Error getting base\r\n\r\n"),
        Ok(hash_base) => hash_base,
    };

    let hash_head = match get_branch_from_repo(head, repo) {
        Err(_) => return Err("400 Error getting head\r\n\r\n"),
        Ok(hash_head) => hash_head,
    };

    Ok(validate_merge(repo, &hash_base, &hash_head).is_ok())
}

pub(crate) fn create_pr(
    body: BodyParameters,
    path: &str,
    repo: &str,
) -> Result<PullRequest, &'static str> {
    println!("Antes de merge_test");
    let mergeable = match merge_test(&body.base, &body.head, repo) {
        Err(e) => return Err(e),
        Ok(res) => res,
    };
    println!("Salio de merge_test");

    let current_time: DateTime<Utc> = Utc::now();
    let formatted_time = current_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let id = match new_pull_request_id(repo) {
        Err(e) => return Err(e),
        Ok(id) => id,
    };

    Ok(PullRequest {
        id,
        head: body.head,
        base: body.base,
        repo: repo.to_string(),
        state: "open".to_string(),
        merged: false,
        mergeable,
        title: body.title,
        html_url: path.to_string(),
        created_at: formatted_time,
    })
}

/// /repos/{repo}/pulls
pub fn handle_post(path: &str, body: String) -> String {
    pullrequest_controller::create_pull_request(path, body)
}

/// receives string and validates if it respects the format /repos/{repo}/pulls
/// returns the repo name on success, ServerError on error
pub fn validate_repo_path(path: &str) -> Result<String, &'static str> {
    let repo_name = path.replace("/repos/", "").replace("/pulls", "") + ".git";
    let path = std::path::Path::new(&repo_name);
    if !path.is_dir() {
        return Err("422 No repo with sugested name exists\r\n\r\n");
    }

    Ok(repo_name)
}
