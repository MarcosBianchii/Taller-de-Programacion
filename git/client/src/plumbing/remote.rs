use super::super::{
    commands::RemoteCommand,
    config_file::{config::Config, config_entry::ConfigEntry},
};
use crate::io_err;
use std::io;

// Underlying implementation of `git remote`.
#[allow(dead_code)]
pub fn __remote(cmd: RemoteCommand, config: &mut Config) -> io::Result<Option<Vec<String>>> {
    use RemoteCommand::*;
    match cmd {
        Add { name, url } => {
            if config.contains_key(&name) {
                return Err(io_err!("Remote already exists"));
            }

            let entry = ConfigEntry::Remote {
                fetch: format!("+refs/heads/*:refs/remotes/{}/*", &name),
                name: name.clone(),
                url,
            };

            config.insert(name, entry);
        }

        Rem { name } => {
            let _ = config.remove(&name);
        }

        List => {
            let mut remotes = config
                .iter()
                .filter(|(_, entry)| entry.is_remote())
                .collect::<Vec<_>>();

            // The structure is the following: (name, entry).
            remotes.sort_by(|a, b| a.1.name().cmp(&b.1.name()));
            return Ok(Some(remotes.into_iter().map(|e| e.1.name()).collect()));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use RemoteCommand::*;

    macro_rules! test_str {
        () => {
            r#"[core]
    repositoryformatversion = 0
    filemode = true
    bare = false
    logallrefupdates = true
[remote "origin"]
    url = https://github.com/taller-1-fiuba-rust/23C2-Los-Krabby-Patty.git
    fetch = +refs/heads/*:refs/remotes/origin/*
[branch "main"]
    remote = origin
    merge = refs/heads/main
[branch "test"]
    remote = origin
    merge = refs/heads/test
"#
        };
    }

    #[test]
    fn list() {
        let s = test_str!();

        let mut config = Config::from_str(s).unwrap();
        match __remote(List, &mut config).unwrap() {
            Some(remotes) => assert_eq!(1, remotes.len()),
            None => panic!("Expected Some"),
        }
    }

    #[test]
    fn add() {
        let s = test_str!();

        let mut config = Config::from_str(s).unwrap();
        match __remote(
            Add {
                name: "fetch".to_string(),
                url: "url".to_string(),
            },
            &mut config,
        )
        .unwrap()
        {
            Some(_) => panic!("Expected None"),
            None => {
                assert_eq!(4, config.len());
                assert!(config.contains_key("fetch"));
            }
        }
    }

    #[test]
    fn rm() {
        let s = test_str!();

        let mut config = Config::from_str(s).unwrap();
        match __remote(
            Rem {
                name: "origin".to_string(),
            },
            &mut config,
        )
        .unwrap()
        {
            Some(_) => panic!("Expected None"),
            None => {
                assert_eq!(2, config.len());
                assert!(!config.contains_key("origin"));
            }
        }
    }
}
