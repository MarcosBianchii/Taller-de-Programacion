use std::str::FromStr;

pub enum GitCommand {
    Init(String), // directory,
    HashObject,
    Add,                     // Commit,
    Clone((String, String)), // (repository name, directory name)
    Fetch((String, String)), // (remote name, branch name)
}

impl FromStr for GitCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            //"init" => Ok(GitCommand::Init(),
            "hash-object" => Ok(GitCommand::HashObject),
            "add" => Ok(GitCommand::Add),
            _ => Err(()),
        }
    }
}
